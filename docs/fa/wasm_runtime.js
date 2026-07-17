/**
 * Zamin-to-JavaScript Transpiler
 *
 * Parses Zamin source code and transpiles it to JavaScript,
 * then executes it in an isolated scope with captured output.
 *
 * This replaces the WASM runtime for environments where
 * the WASM binary hasn't been built yet.
 */
(function(global) {
  'use strict';

  // ---- Tokenizer ----

  var TokenType = {
    // Literals
    Identifier: 'Identifier',
    Number: 'Number',
    String: 'String',
    // Keywords
    Let: 'Let', Const: 'Const', Func: 'Func',
    If: 'If', Else: 'Else', Elif: 'Elif',
    While: 'While', For: 'For', In: 'In', Return: 'Return',
    True: 'True', False: 'False', Nil: 'Nil',
    Match: 'Match',
    Break: 'Break', Continue: 'Continue',
    And: 'And', Or: 'Or', Not: 'Not',
    Import: 'Import', From: 'From', As: 'As', Struct: 'Struct',
    Throw: 'Throw', Try: 'Try', Catch: 'Catch',
    Export: 'Export',
    // Punctuation
    LParen: 'LParen', RParen: 'RParen',
    LBrace: 'LBrace', RBrace: 'RBrace',
    LBracket: 'LBracket', RBracket: 'RBracket',
    Semicolon: 'Semicolon', Comma: 'Comma', Colon: 'Colon',
    Dot: 'Dot', DotDot: 'DotDot', Arrow: 'Arrow',
    // Operators
    Plus: 'Plus', Minus: 'Minus', Star: 'Star', Slash: 'Slash',
    Percent: 'Percent', DoubleStar: 'DoubleStar',
    Assign: 'Assign',
    Eq: 'Eq', Ne: 'Ne', Lt: 'Lt', Gt: 'Gt', Le: 'Le', Ge: 'Ge',
    PlusEq: 'PlusEq', MinusEq: 'MinusEq', StarEq: 'StarEq', SlashEq: 'SlashEq',
    Inc: 'Inc', Dec: 'Dec',
    Newline: 'Newline', Eof: 'Eof',
    Question: 'Question', Pipe: 'Pipe', FatArrow: 'FatArrow',
  };

  var KEYWORDS = {};
  KEYWORDS['let'] = TokenType.Let;
  KEYWORDS['const'] = TokenType.Const;
  KEYWORDS['func'] = TokenType.Func;
  KEYWORDS['if'] = TokenType.If;
  KEYWORDS['else'] = TokenType.Else;
  KEYWORDS['elif'] = TokenType.Elif;
  KEYWORDS['while'] = TokenType.While;
  KEYWORDS['for'] = TokenType.For;
  KEYWORDS['in'] = TokenType.In;
  KEYWORDS['return'] = TokenType.Return;
  KEYWORDS['true'] = TokenType.True;
  KEYWORDS['false'] = TokenType.False;
  KEYWORDS['nil'] = TokenType.Nil;
  KEYWORDS['match'] = TokenType.Match;
  KEYWORDS['break'] = TokenType.Break;
  KEYWORDS['continue'] = TokenType.Continue;
  KEYWORDS['and'] = TokenType.And;
  KEYWORDS['or'] = TokenType.Or;
  KEYWORDS['not'] = TokenType.Not;
  KEYWORDS['import'] = TokenType.Import;
  KEYWORDS['from'] = TokenType.From;
  KEYWORDS['as'] = TokenType.As;
  KEYWORDS['struct'] = TokenType.Struct;
  KEYWORDS['throw'] = TokenType.Throw;
  KEYWORDS['try'] = TokenType.Try;
  KEYWORDS['catch'] = TokenType.Catch;
  KEYWORDS['export'] = TokenType.Export;

  function tokenize(source) {
    var tokens = [];
    var i = 0;
    var line = 1;
    var col = 1;

    function error(msg) {
      throw new Error('Lexer error at ' + line + ':' + col + ': ' + msg);
    }

    function add(type, value) {
      tokens.push({ type: type, value: value !== undefined ? value : null, line: line, col: col });
    }

    function isAlpha(c) {
      return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c === '_';
    }

    function isDigit(c) {
      return c >= '0' && c <= '9';
    }

    function isAlnum(c) {
      return isAlpha(c) || isDigit(c);
    }

    function advance() {
      var c = source[i];
      i++;
      col++;
      return c;
    }

    function peek() {
      return i < source.length ? source[i] : '\0';
    }

    // Look ahead one character WITHOUT advancing
    function lookahead() {
      return i + 1 < source.length ? source[i + 1] : '\0';
    }

    while (i < source.length) {
      var c = source[i];

      // Skip whitespace (except newlines)
      if (c === ' ' || c === '\t' || c === '\r') {
        advance();
        continue;
      }

      // Newline
      if (c === '\n') {
        advance();
        line++;
        col = 1;
        continue;
      }

      // Single-line comment
      if (c === '/' && lookahead() === '/') {
        while (i < source.length && source[i] !== '\n') advance();
        continue;
      }
      // Block comment
      if (c === '/' && lookahead() === '*') {
        advance(); advance();
        while (i < source.length && !(source[i] === '*' && lookahead() === '/')) {
          if (source[i] === '\n') { line++; col = 1; }
          else advance();
        }
        if (i < source.length) { advance(); advance(); }
        continue;
      }

      // Strings
      if (c === '"' || c === "'") {
        var quote = c;
        advance();
        var str = '';
        while (i < source.length && source[i] !== quote) {
          if (source[i] === '\\') {
            advance();
            switch (source[i]) {
              case 'n': str += '\n'; break;
              case 't': str += '\t'; break;
              case 'r': str += '\r'; break;
              case '"': str += '"'; break;
              case "'": str += "'"; break;
              case '\\': str += '\\'; break;
              default: str += source[i]; break;
            }
            advance();
          } else {
            str += advance();
          }
        }
        if (i < source.length) advance();
        add(TokenType.String, str);
        continue;
      }

      // Numbers
      if (isDigit(c) || (c === '.' && isDigit(lookahead()))) {
        var num = c;
        advance();
        if (c === '0' && (peek() === 'x' || peek() === 'X' || peek() === 'b' || peek() === 'B' || peek() === 'o' || peek() === 'O')) {
          num += advance();
          num += advance();
          while (isAlnum(peek())) num += advance();
        } else if (c === '.') {
          while (isDigit(peek())) { num += advance(); }
        } else {
          while (isDigit(peek())) { num += advance(); }
          if (peek() === '.' && isDigit(lookahead())) {
            num += advance();
            while (isDigit(peek())) num += advance();
          }
          if (peek() === 'e' || peek() === 'E') {
            num += advance();
            if (peek() === '+' || peek() === '-') num += advance();
            while (isDigit(peek())) num += advance();
          }
        }
        add(TokenType.Number, num);
        continue;
      }

      // Identifiers and keywords
      if (isAlpha(c)) {
        var id = c;
        advance();
        while (isAlnum(peek())) id += advance();
        var kw = KEYWORDS[id];
        if (kw) {
          add(kw, id);
        } else {
          add(TokenType.Identifier, id);
        }
        continue;
      }

      // Multi-character operators
      var c2 = c + lookahead();
      switch (c2) {
        case '..': advance(); advance(); add(TokenType.DotDot, '..'); continue;
        case '==': advance(); advance(); add(TokenType.Eq, '=='); continue;
        case '!=': advance(); advance(); add(TokenType.Ne, '!='); continue;
        case '<=': advance(); advance(); add(TokenType.Le, '<='); continue;
        case '>=': advance(); advance(); add(TokenType.Ge, '>='); continue;
        case '**': advance(); advance(); add(TokenType.DoubleStar, '**'); continue;
        case '++': advance(); advance(); add(TokenType.Inc, '++'); continue;
        case '--': advance(); advance(); add(TokenType.Dec, '--'); continue;
        case '+=': advance(); advance(); add(TokenType.PlusEq, '+='); continue;
        case '-=': advance(); advance(); add(TokenType.MinusEq, '-='); continue;
        case '*=': advance(); advance(); add(TokenType.StarEq, '*='); continue;
        case '/=': advance(); advance(); add(TokenType.SlashEq, '/='); continue;
        case '->': advance(); advance(); add(TokenType.Arrow, '->'); continue;
        case '||': advance(); advance(); add(TokenType.Pipe, '||'); continue;
        case '=>': advance(); advance(); add(TokenType.FatArrow, '=>'); continue;
        case '//':
          advance(); advance();
          add(TokenType.Slash, '/');
          // The second '/' was already consumed; this is int-div
          // We emit just '/' for now and handle int-div at parse time
          continue;
      }

      // Single-character operators
      switch (c) {
        case '(': advance(); add(TokenType.LParen, '('); continue;
        case ')': advance(); add(TokenType.RParen, ')'); continue;
        case '{': advance(); add(TokenType.LBrace, '{'); continue;
        case '}': advance(); add(TokenType.RBrace, '}'); continue;
        case '[': advance(); add(TokenType.LBracket, '['); continue;
        case ']': advance(); add(TokenType.RBracket, ']'); continue;
        case ';': advance(); add(TokenType.Semicolon, ';'); continue;
        case ',': advance(); add(TokenType.Comma, ','); continue;
        case ':': advance(); add(TokenType.Colon, ':'); continue;
        case '.': advance(); add(TokenType.Dot, '.'); continue;
        case '+': advance(); add(TokenType.Plus, '+'); continue;
        case '-': advance(); add(TokenType.Minus, '-'); continue;
        case '*': advance(); add(TokenType.Star, '*'); continue;
        case '/': advance(); add(TokenType.Slash, '/'); continue;
        case '%': advance(); add(TokenType.Percent, '%'); continue;
        case '=': advance(); add(TokenType.Assign, '='); continue;
        case '<': advance(); add(TokenType.Lt, '<'); continue;
        case '>': advance(); add(TokenType.Gt, '>'); continue;
        case '!': advance(); add(TokenType.Not, '!'); continue;
        case '?': advance(); add(TokenType.Question, '?'); continue;
        case '|': advance(); add(TokenType.Pipe, '|'); continue;
        case '&': advance(); add(TokenType.Not, '&'); continue;
        case '^': advance(); add(TokenType.Not, '^'); continue;
        case '~': advance(); add(TokenType.Not, '~'); continue;
      }

      error("unexpected character '" + c + "'");
    }

    add(TokenType.Eof);
    return tokens;
  }

  // ---- Parser ----

  function Parser(tokens) {
    this.tokens = tokens;
    this.pos = 0;
  }

  Parser.prototype.peek = function() {
    return this.tokens[this.pos] || { type: TokenType.Eof };
  };

  Parser.prototype.peekNext = function() {
    return this.tokens[this.pos + 1] || { type: TokenType.Eof };
  };

  Parser.prototype.advance = function() {
    var t = this.tokens[this.pos];
    if (this.pos < this.tokens.length) this.pos++;
    return t || { type: TokenType.Eof };
  };

  Parser.prototype.expect = function(type) {
    var t = this.peek();
    if (t.type !== type) {
      throw new Error('Parse error at ' + t.line + ':' + t.col + ': expected ' + type + ', got ' + t.type);
    }
    return this.advance();
  };

  Parser.prototype.maybe = function(type) {
    if (this.peek().type === type) {
      return this.advance();
    }
    return null;
  };

  Parser.prototype.parseProgram = function() {
    var stmts = [];
    while (this.peek().type !== TokenType.Eof) {
      stmts.push(this.parseStmt());
      this.maybe(TokenType.Semicolon);
    }
    return { type: 'Program', stmts: stmts };
  };

  Parser.prototype.parseStmt = function() {
    var t = this.peek();

    // let / const
    if (t.type === TokenType.Let || t.type === TokenType.Const) {
      var isConst = t.type === TokenType.Const;
      this.advance();
      var name = this.expect(TokenType.Identifier).value;
      this.expect(TokenType.Assign);
      var value = this.parseExpr();
      this.maybe(TokenType.Semicolon);
      return { type: 'Let', name: name, value: value, isConst: isConst };
    }

    // func
    if (t.type === TokenType.Func) {
      return this.parseFuncDef();
    }

    // if
    if (t.type === TokenType.If) {
      return this.parseIf();
    }

    // while
    if (t.type === TokenType.While) {
      this.advance();
      var cond = this.parseExpr();
      var body = this.parseBlock();
      this.maybe(TokenType.Semicolon);
      return { type: 'While', condition: cond, body: body };
    }

    // for
    if (t.type === TokenType.For) {
      this.advance();
      var varName = this.expect(TokenType.Identifier).value;
      this.expect(TokenType.In);
      var iterable = this.parseExpr();
      var fbody = this.parseBlock();
      this.maybe(TokenType.Semicolon);
      return { type: 'For', variable: varName, iterable: iterable, body: fbody };
    }

    // return
    if (t.type === TokenType.Return) {
      this.advance();
      var val = null;
      if (this.peek().type !== TokenType.Semicolon && this.peek().type !== TokenType.RBrace) {
        val = this.parseExpr();
      }
      this.maybe(TokenType.Semicolon);
      return { type: 'Return', value: val };
    }

    // match
    if (t.type === TokenType.Match) {
      return this.parseMatch();
    }

    // break / continue
    if (t.type === TokenType.Break) { this.advance(); this.maybe(TokenType.Semicolon); return { type: 'Break' }; }
    if (t.type === TokenType.Continue) { this.advance(); this.maybe(TokenType.Semicolon); return { type: 'Continue' }; }

    // import
    if (t.type === TokenType.Import) {
      this.advance();
      var modName = this.expect(TokenType.Identifier).value;
      var alias = null;
      if (this.maybe(TokenType.As)) {
        alias = this.expect(TokenType.Identifier).value;
      }
      this.maybe(TokenType.Semicolon);
      return { type: 'Import', module: modName, alias: alias };
    }

    // struct
    if (t.type === TokenType.Struct) {
      this.advance();
      var structName = this.expect(TokenType.Identifier).value;
      this.expect(TokenType.LBrace);
      var methods = [];
      while (this.peek().type !== TokenType.RBrace && this.peek().type !== TokenType.Eof) {
        methods.push(this.parseStmt());
      }
      this.expect(TokenType.RBrace);
      this.maybe(TokenType.Semicolon);
      return { type: 'StructDef', name: structName, methods: methods };
    }

    // try / throw - minimal support
    if (t.type === TokenType.Throw) {
      this.advance();
      var exc = this.parseExpr();
      this.maybe(TokenType.Semicolon);
      return { type: 'Throw', value: exc };
    }

    if (t.type === TokenType.Try) {
      this.advance();
      var tryBody = this.parseBlock();
      var catchVar = null;
      var catchBody = null;
      if (this.maybe(TokenType.Catch)) {
        catchVar = this.expect(TokenType.Identifier).value;
        catchBody = this.parseBlock();
      }
      this.maybe(TokenType.Semicolon);
      return { type: 'Try', body: tryBody, catchVar: catchVar, catchBody: catchBody };
    }

    // export
    if (t.type === TokenType.Export) {
      this.advance();
      this.expect(TokenType.LBrace);
      var names = [];
      while (this.peek().type !== TokenType.RBrace && this.peek().type !== TokenType.Eof) {
        names.push(this.expect(TokenType.Identifier).value);
        this.maybe(TokenType.Comma);
      }
      this.expect(TokenType.RBrace);
      this.maybe(TokenType.Semicolon);
      return { type: 'Export', names: names };
    }

    // Block (standalone { })
    if (t.type === TokenType.LBrace) {
      var block = this.parseBlock();
      this.maybe(TokenType.Semicolon);
      return { type: 'Block', body: block };
    }

    // Expression statement
    var expr = this.parseExpr();
    this.maybe(TokenType.Semicolon);
    return { type: 'Expr', expression: expr };
  };

  Parser.prototype.parseBlock = function() {
    this.expect(TokenType.LBrace);
    var stmts = [];
    while (this.peek().type !== TokenType.RBrace && this.peek().type !== TokenType.Eof) {
      stmts.push(this.parseStmt());
      this.maybe(TokenType.Semicolon);
    }
    this.expect(TokenType.RBrace);
    return stmts;
  };

  Parser.prototype.parseFuncDef = function() {
    this.advance(); // func
    var name = this.expect(TokenType.Identifier).value;
    this.expect(TokenType.LParen);
    var params = [];
    while (this.peek().type !== TokenType.RParen && this.peek().type !== TokenType.Eof) {
      params.push(this.expect(TokenType.Identifier).value);
      this.maybe(TokenType.Comma);
    }
    this.expect(TokenType.RParen);
    var body = this.parseBlock();
    this.maybe(TokenType.Semicolon);
    return { type: 'FuncDef', name: name, params: params, body: body };
  };

  Parser.prototype.parseIf = function() {
    this.advance(); // if
    var cond = this.parseExpr();
    var thenBranch = this.parseBlock();
    var elifBranches = [];
    var elseBranch = null;
    while (this.peek().type === TokenType.Elif) {
      this.advance();
      var econd = this.parseExpr();
      var ebody = this.parseBlock();
      elifBranches.push({ condition: econd, body: ebody });
    }
    if (this.peek().type === TokenType.Else) {
      this.advance();
      elseBranch = this.parseBlock();
    }
    this.maybe(TokenType.Semicolon);
    return { type: 'If', condition: cond, thenBranch: thenBranch, elifBranches: elifBranches, elseBranch: elseBranch };
  };

  Parser.prototype.parseMatch = function() {
    this.advance(); // match
    var value = this.parseExpr();
    this.expect(TokenType.LBrace);
    var arms = [];
    while (this.peek().type !== TokenType.RBrace && this.peek().type !== TokenType.Eof) {
      var pattern = this.parseExpr();
      this.expect(TokenType.FatArrow);
      var bodyStmts = [];
      // Body can be a single expression or a block
      if (this.peek().type === TokenType.LBrace) {
        bodyStmts = this.parseBlock();
      } else {
        bodyStmts.push({ type: 'Expr', expression: this.parseExpr() });
      }
      arms.push({ pattern: pattern, body: bodyStmts });
      this.maybe(TokenType.Comma);
    }
    this.expect(TokenType.RBrace);
    this.maybe(TokenType.Semicolon);
    return { type: 'Match', value: value, arms: arms };
  };

  // ---- Expression Parser (precedence climbing) ----

  var PREC = {
    LOWEST: 1,
    ASSIGN: 2,
    OR: 3,
    AND: 4,
    EQ: 5,
    CMP: 6,
    RANGE: 7,
    TERM: 8,
    FACTOR: 9,
    UNARY: 10,
    CALL: 11,
    PRIMARY: 12,
  };

  var PREFIX_OPS = {};
  PREFIX_OPS[TokenType.Plus] = { prec: PREC.UNARY };
  PREFIX_OPS[TokenType.Minus] = { prec: PREC.UNARY };
  PREFIX_OPS[TokenType.Not] = { prec: PREC.UNARY };
  PREFIX_OPS[TokenType.Inc] = { prec: PREC.UNARY };
  PREFIX_OPS[TokenType.Dec] = { prec: PREC.UNARY };

  var INFIX_OPS = {};
  INFIX_OPS[TokenType.Plus] = { prec: PREC.TERM, left: true };
  INFIX_OPS[TokenType.Minus] = { prec: PREC.TERM, left: true };
  INFIX_OPS[TokenType.Star] = { prec: PREC.FACTOR, left: true };
  INFIX_OPS[TokenType.Slash] = { prec: PREC.FACTOR, left: true };
  INFIX_OPS[TokenType.Percent] = { prec: PREC.FACTOR, left: true };
  INFIX_OPS[TokenType.DoubleStar] = { prec: PREC.FACTOR, left: false };
  INFIX_OPS[TokenType.Eq] = { prec: PREC.EQ, left: true };
  INFIX_OPS[TokenType.Ne] = { prec: PREC.EQ, left: true };
  INFIX_OPS[TokenType.Lt] = { prec: PREC.CMP, left: true };
  INFIX_OPS[TokenType.Gt] = { prec: PREC.CMP, left: true };
  INFIX_OPS[TokenType.Le] = { prec: PREC.CMP, left: true };
  INFIX_OPS[TokenType.Ge] = { prec: PREC.CMP, left: true };
  INFIX_OPS[TokenType.And] = { prec: PREC.AND, left: true };
  INFIX_OPS[TokenType.Or] = { prec: PREC.OR, left: true };
  INFIX_OPS[TokenType.Dot] = { prec: PREC.CALL, left: true };
  INFIX_OPS[TokenType.Assign] = { prec: PREC.ASSIGN, left: false };
  INFIX_OPS[TokenType.PlusEq] = { prec: PREC.ASSIGN, left: false };
  INFIX_OPS[TokenType.MinusEq] = { prec: PREC.ASSIGN, left: false };
  INFIX_OPS[TokenType.StarEq] = { prec: PREC.ASSIGN, left: false };
  INFIX_OPS[TokenType.SlashEq] = { prec: PREC.ASSIGN, left: false };
  INFIX_OPS[TokenType.DotDot] = { prec: PREC.RANGE, left: true };

  // Also handle ++ and -- as prefix/postfix? We'll do prefix for now.

  Parser.prototype.parseExpr = function(prec) {
    if (prec === undefined) prec = PREC.LOWEST;
    var t = this.peek();
    var left = null;

    // Prefix
    if (t.type === TokenType.Plus || t.type === TokenType.Minus || t.type === TokenType.Not ||
        t.type === TokenType.Inc || t.type === TokenType.Dec) {
      var op = this.advance();
      var operand = this.parseExpr(PREC.UNARY);
      left = { type: 'Unary', op: op.value || op.type, operand: operand };
    }
    // Literal / Primary
    else if (t.type === TokenType.Number) {
      this.advance();
      left = { type: 'Number', value: t.value };
    }
    else if (t.type === TokenType.String) {
      this.advance();
      left = { type: 'String', value: t.value };
    }
    else if (t.type === TokenType.True) { this.advance(); left = { type: 'Bool', value: true }; }
    else if (t.type === TokenType.False) { this.advance(); left = { type: 'Bool', value: false }; }
    else if (t.type === TokenType.Nil) { this.advance(); left = { type: 'Nil' }; }
    else if (t.type === TokenType.Identifier) {
      this.advance();
      left = { type: 'Identifier', name: t.value };
    }
    else if (t.type === TokenType.LParen) {
      this.advance();
      left = this.parseExpr();
      this.expect(TokenType.RParen);
    }
    else if (t.type === TokenType.LBracket) {
      this.advance();
      var items = [];
      while (this.peek().type !== TokenType.RBracket && this.peek().type !== TokenType.Eof) {
        items.push(this.parseExpr());
        this.maybe(TokenType.Comma);
      }
      this.expect(TokenType.RBracket);
      left = { type: 'List', items: items };
    }
    else if (t.type === TokenType.LBrace) {
      // Could be dict or block. If followed by a newline/another brace, it's block handled in stmt.
      // As an expression, it's a dict literal.
      this.advance();
      var entries = [];
      while (this.peek().type !== TokenType.RBrace && this.peek().type !== TokenType.Eof) {
        var key = this.parseExpr();
        if (this.maybe(TokenType.Colon)) {
          var val = this.parseExpr();
          entries.push({ key: key, value: val });
        } else {
          // Set literal
          entries.push({ key: key, value: null });
        }
        this.maybe(TokenType.Comma);
      }
      this.expect(TokenType.RBrace);
      left = { type: 'Dict', entries: entries };
    }
    else if (t.type === TokenType.Func) {
      // Anonymous function / lambda (simplified)
      this.advance();
      this.expect(TokenType.LParen);
      var params = [];
      while (this.peek().type !== TokenType.RParen && this.peek().type !== TokenType.Eof) {
        params.push(this.expect(TokenType.Identifier).value);
        this.maybe(TokenType.Comma);
      }
      this.expect(TokenType.RParen);
      var body = this.parseBlock();
      left = { type: 'Func', name: null, params: params, body: body };
    }
    else if (t.type === TokenType.Match) {
      // Match as an expression (value-producing)
      var mexpr = this.parseMatch();
      // parseMatch already calls this.maybe(Semicolon) - we want it as expression
      // Override: if the result is a single-expression body per arm, we keep it as Expr type
      if (left === null) {
        left = mexpr;
      } else {
        left = mexpr;
      }
    }
    else {
      throw new Error('Parse error at ' + t.line + ':' + t.col + ': unexpected token ' + t.type + (t.value ? ' (' + t.value + ')' : ''));
    }

    // Infix / postfix
    while (prec < this.infixPrec(this.peek().type)) {
      t = this.peek();

      // Call
      if (t.type === TokenType.LParen) {
        this.advance();
        var args = [];
        while (this.peek().type !== TokenType.RParen && this.peek().type !== TokenType.Eof) {
          args.push(this.parseExpr());
          this.maybe(TokenType.Comma);
        }
        this.expect(TokenType.RParen);
        left = { type: 'Call', callee: left, args: args };
        continue;
      }

      // Index
      if (t.type === TokenType.LBracket) {
        this.advance();
        var index = this.parseExpr();
        this.expect(TokenType.RBracket);
        left = { type: 'Index', obj: left, index: index };
        continue;
      }

      // Dot access
      if (t.type === TokenType.Dot) {
        this.advance();
        var prop = this.expect(TokenType.Identifier).value;
        // Check if followed by LParen -> method call
        if (this.peek().type === TokenType.LParen) {
          this.advance();
          var margs = [];
          while (this.peek().type !== TokenType.RParen && this.peek().type !== TokenType.Eof) {
            margs.push(this.parseExpr());
            this.maybe(TokenType.Comma);
          }
          this.expect(TokenType.RParen);
          left = { type: 'MethodCall', obj: left, method: prop, args: margs };
          continue;
        }
        left = { type: 'Attr', obj: left, name: prop };
        continue;
      }

      // Infix operators
      var opInfo = INFIX_OPS[t.type];
      if (opInfo) {
        this.advance();
        var op = t.value || t.type;
        var nextPrec = opInfo.prec + (opInfo.left ? 0 : 1);
        var right = this.parseExpr(nextPrec);
        left = { type: 'Binary', op: op, left: left, right: right };
        continue;
      }

      break;
    }

    return left;
  };

  Parser.prototype.infixPrec = function(type) {
    var info = INFIX_OPS[type];
    if (info) return info.prec;
    if (type === TokenType.LParen || type === TokenType.LBracket) return PREC.CALL;
    return 0;
  };

  // ---- Transpiler: Zamin AST -> JavaScript string ----

  var BUILTIN_STDLIB = {
    math: {
      sqrt: 'Math.sqrt', pow: 'Math.pow', abs: 'Math.abs',
      floor: 'Math.floor', ceil: 'Math.ceil', round: 'Math.round',
      sin: 'Math.sin', cos: 'Math.cos', tan: 'Math.tan',
      log: 'Math.log', log10: 'Math.log10',
      pi: '() => Math.PI', e: '() => Math.E',
      inf: '() => Infinity', nan: '() => NaN',
      matrix_add: null, det: null,
    },
    random: {
      int: null, float: null, choice: null,
    },
    json: {
      stringify: 'JSON.stringify', parse: 'JSON.parse',
    },
    string: {
      len: null, upper: null, lower: null, trim: null, split: null,
      contains: null, starts_with: null, ends_with: null, replace: null, repeat: null,
    },
    re: { match: null, find: null, replace: null },
    datetime: { now: null, format: null },
    hashlib: { sha256: null, md5: null },
    base64: { encode: null, decode: null },
    url: { encode: null, decode: null, parse: null },
    io: { print: null, println: null, input: null },
    fs: { read: null, write: null, exists: null, mkdir: null },
    os: { cwd: null, args: null, getenv: null, name: null, system: null },
    csv: { parse: null, serialize: null },
    collections: { len: null },
    comet: null,
  };

  function transpile(node) {
    if (!node) return 'null';

    switch (node.type) {
      case 'Program':
        return node.stmts.map(transpile).join(';\n');

      case 'Expr':
        // Expression statement - ensure it doesn't produce value in console
        return transpile(node.expression);

      case 'Let':
        var kw = node.isConst ? 'const' : 'let';
        return kw + ' ' + node.name + ' = ' + transpile(node.value);

      case 'FuncDef':
        return 'function ' + node.name + '(' + node.params.join(',') + ') {\n' +
          node.body.map(transpile).join(';\n') + '\n}';

      case 'Func':
        return '(function(' + node.params.join(',') + ') {\n' +
          node.body.map(transpile).join(';\n') + '\n})';

      case 'If':
        var js = 'if (' + transpile(node.condition) + ') {\n' +
          node.thenBranch.map(transpile).join(';\n') + '\n}';
        for (var i = 0; i < node.elifBranches.length; i++) {
          js += ' else if (' + transpile(node.elifBranches[i].condition) + ') {\n' +
            node.elifBranches[i].body.map(transpile).join(';\n') + '\n}';
        }
        if (node.elseBranch) {
          js += ' else {\n' + node.elseBranch.map(transpile).join(';\n') + '\n}';
        }
        return js;

      case 'While':
        return 'while (' + transpile(node.condition) + ') {\n' +
          node.body.map(transpile).join(';\n') + '\n}';

      case 'For':
        // for x in expr -> iterate over expr as array
        var iterName = '__iter_' + node.variable;
        var idxName = '__idx_' + node.variable;
        return '(function() { var ' + iterName + ' = ' + transpile(node.iterable) + ';' +
          'if (typeof ' + iterName + ' === "number") {' +
          '  ' + iterName + ' = Array.from({length: ' + iterName + '}, function(_,i){return i;});' +
          '}' +
          'for (var ' + idxName + ' = 0; ' + idxName + ' < ' + iterName + '.length; ' + idxName + '++) {' +
          'var ' + node.variable + ' = ' + iterName + '[' + idxName + '];' +
          node.body.map(transpile).join(';\n') +
          '}})()';

      case 'Return':
        if (node.value) return 'return ' + transpile(node.value);
        return 'return null';

      case 'Break': return 'break';
      case 'Continue': return 'continue';

      case 'Block':
        return '{\n' + node.body.map(transpile).join(';\n') + '\n}';

      case 'Match':
        var mval = transpile(node.value);
        var arms = node.arms.map(function(a) {
          var pat = transpile(a.pattern);
          var body = a.body.map(transpile).join(';\n');
          return 'if (' + mval + ' === ' + pat + ') { return ' + body + '; }';
        });
        return '(function() { ' + arms.join(' else ') + ' })()';

      case 'Import':
        var mod = node.module;
        var alias = node.alias || mod;
        return 'var ' + alias + ' = __builtinModules["' + mod + '"] || {};';

      case 'StructDef':
        // struct -> class with methods
        var methods = node.methods.map(function(m) {
          if (m.type === 'FuncDef') {
            return m.name + ': function(' + m.params.join(',') + ') {\n' +
              m.body.map(transpile).join(';\n') + '\n}';
          }
          return '';
        }).filter(Boolean);
        return 'function ' + node.name + '() {} ' +
          node.name + '.prototype = {\n' + methods.join(',\n') + '\n}';

      case 'Try':
        return 'try {\n' + node.body.map(transpile).join(';\n') + '\n} catch (' +
          (node.catchVar || 'e') + ') {\n' + (node.catchBody ? node.catchBody.map(transpile).join(';\n') : '') + '\n}';

      case 'Throw':
        return 'throw ' + transpile(node.value);

      // Expressions
      case 'Number':
        return node.value;

      case 'String':
        return JSON.stringify(node.value);

      case 'Bool':
        return node.value ? 'true' : 'false';

      case 'Nil':
        return 'null';

      case 'Identifier':
        return node.name;

      case 'Unary':
        if (node.op === 'not' || node.op === '!') return '!' + transpile(node.operand);
        if (node.op === '-') return '-' + transpile(node.operand);
        if (node.op === '+') return '+' + transpile(node.operand);
        if (node.op === '++') return '(' + transpile(node.operand) + ' + 1)';
        if (node.op === '--') return '(' + transpile(node.operand) + ' - 1)';
        return node.op + transpile(node.operand);

      case 'Binary':
        if (node.op === '..') {
          return 'Array.from({length: Math.max(0, (' + transpile(node.right) + ' - ' + transpile(node.left) + '))}, function(_,i){return i + ' + transpile(node.left) + ';})';
        }
        return '(' + transpile(node.left) + ' ' + jsOp(node.op) + ' ' + transpile(node.right) + ')';

      case 'Call':
        var callee = transpile(node.callee);
        var args = node.args.map(transpile).join(',');
        return callee + '(' + args + ')';

      case 'MethodCall':
        var obj = transpile(node.obj);
        var margs = node.args.map(transpile).join(',');
        // Handle special built-in methods
        if (obj === '__io') {
          if (node.method === 'print') return '__print(' + margs + ')';
          if (node.method === 'println') return '__println(' + margs + ')';
        }
        if (obj === '__math') {
          return '__mathCall("' + node.method + '", [' + margs + '])';
        }
        if (obj === '__random') {
          return '__randomCall("' + node.method + '", [' + margs + '])';
        }
        return obj + '.' + node.method + '(' + margs + ')';

      case 'Attr':
        return transpile(node.obj) + '.' + node.name;

      case 'Index':
        return transpile(node.obj) + '[' + transpile(node.index) + ']';

      case 'List':
        return '[' + node.items.map(transpile).join(',') + ']';

      case 'Dict':
        var entries = node.entries.map(function(e) {
          if (e.value !== null) {
            return transpile(e.key) + ': ' + transpile(e.value);
          }
          // Set literal entry
          return transpile(e.key);
        });
        return '{' + entries.join(',') + '}';
    }

    return 'null';
  }

  function jsOp(op) {
    switch (op) {
      case 'and': case '&&': return '&&';
      case 'or': case '||': return '||';
      case 'not': case '!': return '!';
      case '==': return '===';
      case '!=': return '!==';
      case '..': case '...': return '..'; // range - handled specially
      case 'Assign': return '=';
      case '+=': return '+=';
      case '-=': return '-=';
      case '*=': return '*=';
      case '/=': return '/=';
      case '**': return '**';
      case '.': return '.';
    }
    return op;
  }

  // ---- Helper for built-in module calls ----

  // ---- Main execute function ----

  function executeZamin(sourceCode) {
    var outputBuf = { text: '' };

    try {
      var tokens = tokenize(sourceCode);
      var parser = new Parser(tokens);
      var ast = parser.parseProgram();

      // Build the JS code
      var jsCode = transpile(ast);

      // Set up scope with builtins
      var __builtinModules = {
        math: {
          sqrt: Math.sqrt, pow: Math.pow, abs: Math.abs,
          floor: Math.floor, ceil: Math.ceil,           round: function(x, n) { return n !== undefined ? Math.round(x * Math.pow(10, n)) / Math.pow(10, n) : Math.round(x); },
          sin: Math.sin, cos: Math.cos, tan: Math.tan,
          log: Math.log, log10: Math.log10,
          pi: Math.PI, e: Math.E, inf: Infinity, nan: NaN,
          matrix_add: function(a, b) {
            return a.map(function(row, i) { return row.map(function(val, j) { return val + b[i][j]; }); });
          },
          det: function(m) {
            if (m.length === 2 && m[0].length === 2) return m[0][0] * m[1][1] - m[0][1] * m[1][0];
            return 0;
          },
        },
        random: {
          int: function(min, max) { return Math.floor(Math.random() * (max - min + 1)) + min; },
          float: function() { return Math.random(); },
          choice: function(arr) { return arr[Math.floor(Math.random() * arr.length)]; },
        },
        json: {
          stringify: JSON.stringify,
          parse: JSON.parse,
        },
        io: {
          print: function(s) { outputBuf.text += String(s); },
          println: function(s) { outputBuf.text += String(s) + '\n'; },
          input: function() { return ''; },
        },
        collections: {
          len: function(x) {
            if (typeof x === 'string' || Array.isArray(x)) return x.length;
            if (typeof x === 'object' && x !== null) return Object.keys(x).length;
            return 0;
          },
        },
        http: {
          get: function(url) { return { status: 200, body: '{}', headers: {} }; },
          post: function(url, body) { return { status: 200, body: '{}', headers: {} }; },
        },
        fs: {
          read: function(path) { return ''; },
          write: function(path, data) { return true; },
          exists: function(path) { return false; },
          mkdir: function(path) { return true; },
        },
        os: {
          cwd: function() { return '/'; },
          args: function() { return []; },
          getenv: function(k) { return null; },
          name: 'browser',
          system: function(cmd) { return 0; },
        },
        string: {
          len: function(s) { return s.length; },
          upper: function(s) { return s.toUpperCase(); },
          lower: function(s) { return s.toLowerCase(); },
          trim: function(s) { return s.trim(); },
          split: function(s, sep) { return s.split(sep); },
          contains: function(s, sub) { return s.indexOf(sub) >= 0; },
          starts_with: function(s, sub) { return s.startsWith(sub); },
          ends_with: function(s, sub) { return s.endsWith(sub); },
          replace: function(s, from, to) { return s.replace(from, to); },
          repeat: function(s, n) { return s.repeat(n); },
        },
        csv: {
          parse: function(s) { return s.split('\n').map(function(l) { return l.split(','); }); },
          serialize: function(rows) { return rows.map(function(r) { return r.join(','); }).join('\n'); },
        },
        re: {
          match: function(pattern, s) { return s.match(new RegExp(pattern)); },
          find: function(pattern, s) { return s.match(new RegExp(pattern)); },
          replace: function(pattern, s, repl) { return s.replace(new RegExp(pattern), repl); },
        },
        datetime: {
          now: function() { return new Date().toISOString(); },
          format: function(fmt, ts) { return new Date(ts || Date.now()).toISOString(); },
        },
        hashlib: {
          sha256: function(s) { return s; },
          md5: function(s) { return s; },
        },
        base64: {
          encode: function(s) { return btoa ? btoa(s) : Buffer.from(s).toString('base64'); },
          decode: function(s) { return atob ? atob(s) : Buffer.from(s, 'base64').toString(); },
        },
        url: {
          encode: encodeURIComponent,
          decode: decodeURIComponent,
          parse: function(url) { var u = new URL(url); return { protocol: u.protocol, host: u.host, pathname: u.pathname, search: u.search, hash: u.hash }; },
        },
      };

      // Wrap in an IIFE with the scope
      var fullCode = '(function() {\n' +
        'var __print = function(s) { outputBuf.text += String(s); };\n' +
        'var __println = function(s) { outputBuf.text += String(s) + "\\n"; };\n' +
        'var print = __print;\n' +
        'var println = __println;\n' +
        'var __io = __builtinModules.io;\n' +
        'var __math = __builtinModules.math;\n' +
        'var __random = __builtinModules.random;\n' +
        'var __json = __builtinModules.json;\n' +
        'var __collections = __builtinModules.collections;\n' +
        'var __scope = {};\n' +
        'var len = __collections.len;\n' +
        jsCode + '\n' +
        '})();\n';

      // Execute
      var fn = new Function('outputBuf', '__builtinModules', fullCode);
      fn(outputBuf, __builtinModules);

      return { success: true, output: outputBuf.text };

    } catch (e) {
      var msg = e.message || String(e);
      if (outputBuf.text) {
        return { success: false, output: outputBuf.text + '\nError: ' + msg };
      }
      return { success: false, output: 'Error: ' + msg };
    }
  }

  // Export
  global.zaminWasm = {
    run_zamin: function(source) {
      var result = executeZamin(source);
      return result.output || '';
    },
    _execute: executeZamin,
  };

})(typeof window !== 'undefined' ? window : this);
