#include "lion.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <math.h>

#define MAX_POOL 64
#define MAX_DIMS 4

static double pool[MAX_POOL * MAX_POOL];
static double work[MAX_POOL * MAX_POOL];

static int parse_flat(const char* s, int len, double* out, int max) {
    int n = 0;
    int i = 0;
    while (i < len && n < max) {
        while (i < len && (s[i] == ' ' || s[i] == ',')) i++;
        if (i >= len) break;
        char buf[64];
        int j = 0;
        while (i < len && s[i] != ' ' && s[i] != ',' && s[i] != ';' && j < 63) buf[j++] = s[i++];
        buf[j] = '\0';
        out[n++] = atof(buf);
    }
    return n;
}

static int count_elements(const char* s, int len) {
    int n = 0, in = 0;
    for (int i = 0; i < len; i++) {
        if (s[i] != ' ' && s[i] != ',' && s[i] != ';' && !in) { n++; in = 1; }
        if (s[i] == ' ' || s[i] == ',' || s[i] == ';') in = 0;
    }
    return n;
}

static int count_rows(const char* s, int len) {
    if (len == 0) return 0;
    int rows = 1;
    for (int i = 0; i < len; i++) if (s[i] == ';') rows++;
    return rows;
}

static int cols_from_row(const char* s, int len) {
    int n = 0, in = 0;
    for (int i = 0; i < len && s[i] != ';'; i++) {
        if (s[i] != ' ' && s[i] != ',' && !in) { n++; in = 1; }
        if (s[i] == ' ' || s[i] == ',') in = 0;
    }
    return n;
}

static LionValue make_str(const char* data) {
    int len = strlen(data);
    static char buf[16384];
    int cp = len < 16383 ? len : 16383;
    memcpy(buf, data, cp);
    buf[cp] = '\0';
    LionValue r;
    r.tag = LION_STRING;
    r.data.as_str.ptr = (const uint8_t*)buf;
    r.data.as_str.len = cp;
    return r;
}

static LionValue make_int(int64_t v) {
    LionValue r;
    r.tag = LION_INT;
    r.data.as_int = v;
    return r;
}

static LionValue make_float(double v) {
    LionValue r;
    r.tag = LION_FLOAT;
    r.data.as_float = v;
    return r;
}

static LionValue make_nil() {
    LionValue r;
    r.tag = LION_NIL;
    return r;
}

static const char* get_str(int argc, const LionValue* args, int i, int* out_len) {
    if (i >= argc || args[i].tag != LION_STRING) { *out_len = 0; return ""; }
    *out_len = (int)args[i].data.as_str.len;
    return (const char*)args[i].data.as_str.ptr;
}

static double get_num(int argc, const LionValue* args, int i, double def) {
    if (i >= argc) return def;
    if (args[i].tag == LION_INT) return (double)args[i].data.as_int;
    if (args[i].tag == LION_FLOAT) return args[i].data.as_float;
    return def;
}

static int get_int(int argc, const LionValue* args, int i, int def) {
    if (i >= argc) return def;
    if (args[i].tag == LION_INT) return (int)args[i].data.as_int;
    return def;
}

static LionValue panda_arange(int argc, const LionValue* args) {
    double start = get_num(argc, args, 0, 0.0);
    double end = get_num(argc, args, 1, 1.0);
    double step = get_num(argc, args, 2, 1.0);
    if (step == 0) return make_nil();
    int n = (int)ceil((end - start) / step);
    if (n > MAX_POOL) n = MAX_POOL;
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++) {
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", start + i * step);
    }
    if (pos > 0) buf[pos - 1] = '\0';
    else buf[0] = '\0';
    return make_str(buf);
}

static LionValue panda_zeros(int argc, const LionValue* args) {
    int n = get_int(argc, args, 0, 1);
    if (n > MAX_POOL) n = MAX_POOL;
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "0 ");
    if (pos > 0) buf[pos - 1] = '\0';
    else buf[0] = '\0';
    return make_str(buf);
}

static LionValue panda_ones(int argc, const LionValue* args) {
    int n = get_int(argc, args, 0, 1);
    if (n > MAX_POOL) n = MAX_POOL;
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "1 ");
    if (pos > 0) buf[pos - 1] = '\0';
    else buf[0] = '\0';
    return make_str(buf);
}

static LionValue panda_linspace(int argc, const LionValue* args) {
    double a = get_num(argc, args, 0, 0.0);
    double b = get_num(argc, args, 1, 1.0);
    int n = get_int(argc, args, 2, 50);
    if (n > MAX_POOL) n = MAX_POOL;
    if (n < 2) n = 2;
    double step = (b - a) / (n - 1);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", a + i * step);
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_sum(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    if (n == 0) return make_float(0);
    parse_flat(s, len, pool, MAX_POOL);
    double total = 0;
    for (int i = 0; i < n; i++) total += pool[i];
    return make_float(total);
}

static LionValue panda_mean(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    if (n == 0) return make_float(0);
    parse_flat(s, len, pool, MAX_POOL);
    double total = 0;
    for (int i = 0; i < n; i++) total += pool[i];
    return make_float(total / n);
}

static LionValue panda_min(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    if (n == 0) return make_float(0);
    parse_flat(s, len, pool, MAX_POOL);
    double v = pool[0];
    for (int i = 1; i < n; i++) if (pool[i] < v) v = pool[i];
    return make_float(v);
}

static LionValue panda_max(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    if (n == 0) return make_float(0);
    parse_flat(s, len, pool, MAX_POOL);
    double v = pool[0];
    for (int i = 1; i < n; i++) if (pool[i] > v) v = pool[i];
    return make_float(v);
}

static LionValue panda_abs(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    if (n == 0) return make_str("");
    parse_flat(s, len, pool, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", fabs(pool[i]));
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_sin(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    parse_flat(s, len, pool, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", sin(pool[i]));
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_cos(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    parse_flat(s, len, pool, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", cos(pool[i]));
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_sqrt(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    parse_flat(s, len, pool, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", sqrt(pool[i]));
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_pow(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    double power = get_num(argc, args, 1, 1.0);
    int n = count_elements(s, len);
    parse_flat(s, len, pool, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", pow(pool[i], power));
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_add(int argc, const LionValue* args) {
    int l1, l2;
    const char* s1 = get_str(argc, args, 0, &l1);
    const char* s2 = get_str(argc, args, 1, &l2);
    int n1 = count_elements(s1, l1);
    int n2 = count_elements(s2, l2);
    int n = n1 < n2 ? n1 : n2;
    if (n == 0) return make_str("");
    parse_flat(s1, l1, pool, MAX_POOL);
    parse_flat(s2, l2, work, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", pool[i] + work[i]);
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_sub(int argc, const LionValue* args) {
    int l1, l2;
    const char* s1 = get_str(argc, args, 0, &l1);
    const char* s2 = get_str(argc, args, 1, &l2);
    int n1 = count_elements(s1, l1);
    int n2 = count_elements(s2, l2);
    int n = n1 < n2 ? n1 : n2;
    if (n == 0) return make_str("");
    parse_flat(s1, l1, pool, MAX_POOL);
    parse_flat(s2, l2, work, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", pool[i] - work[i]);
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_mul(int argc, const LionValue* args) {
    int l1, l2;
    const char* s1 = get_str(argc, args, 0, &l1);
    const char* s2 = get_str(argc, args, 1, &l2);
    int n1 = count_elements(s1, l1);
    int n2 = count_elements(s2, l2);
    int n = n1 < n2 ? n1 : n2;
    if (n == 0) return make_str("");
    parse_flat(s1, l1, pool, MAX_POOL);
    parse_flat(s2, l2, work, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++)
        pos += snprintf(buf + pos, 16384 - pos, "%.10g ", pool[i] * work[i]);
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_dot(int argc, const LionValue* args) {
    int l1, l2;
    const char* s1 = get_str(argc, args, 0, &l1);
    const char* s2 = get_str(argc, args, 1, &l2);
    int n1 = count_elements(s1, l1);
    int n2 = count_elements(s2, l2);
    int n = n1 < n2 ? n1 : n2;
    if (n == 0) return make_float(0);
    parse_flat(s1, l1, pool, MAX_POOL);
    parse_flat(s2, l2, work, MAX_POOL);
    double total = 0;
    for (int i = 0; i < n; i++) total += pool[i] * work[i];
    return make_float(total);
}

static LionValue panda_std(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int n = count_elements(s, len);
    if (n < 2) return make_float(0);
    parse_flat(s, len, pool, MAX_POOL);
    double mean = 0;
    for (int i = 0; i < n; i++) mean += pool[i];
    mean /= n;
    double var = 0;
    for (int i = 0; i < n; i++) var += (pool[i] - mean) * (pool[i] - mean);
    return make_float(sqrt(var / (n - 1)));
}

static LionValue panda_shape(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    if (len == 0) return make_str("(0,)");
    int rows = count_rows(s, len);
    int cols = cols_from_row(s, len);
    static char buf[64];
    if (rows == 1)
        snprintf(buf, 63, "(%d,)", cols);
    else
        snprintf(buf, 63, "(%d, %d)", rows, cols);
    return make_str(buf);
}

static LionValue panda_reshape(int argc, const LionValue* args) {
    int len;
    const char* s = get_str(argc, args, 0, &len);
    int r = get_int(argc, args, 1, 1);
    int c = get_int(argc, args, 2, -1);
    int n = count_elements(s, len);
    if (c < 0) c = n / r;
    if (r * c > MAX_POOL * MAX_POOL) return make_str("");
    parse_flat(s, len, pool, MAX_POOL);
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < r && pos < 16380; i++) {
        for (int j = 0; j < c && pos < 16380; j++) {
            int idx = i * c + j;
            if (idx < n)
                pos += snprintf(buf + pos, 16384 - pos, "%.10g ", pool[idx]);
            else
                pos += snprintf(buf + pos, 16384 - pos, "0 ");
        }
        buf[pos - 1] = ';';
    }
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_eye(int argc, const LionValue* args) {
    int n = get_int(argc, args, 0, 3);
    if (n > MAX_POOL) n = MAX_POOL;
    static char buf[16384];
    int pos = 0;
    for (int i = 0; i < n && pos < 16380; i++) {
        for (int j = 0; j < n && pos < 16380; j++)
            pos += snprintf(buf + pos, 16384 - pos, "%g ", (i == j) ? 1.0 : 0.0);
        buf[pos - 1] = ';';
    }
    if (pos > 0) buf[pos - 1] = '\0';
    return make_str(buf);
}

static LionValue panda_version(int argc, const LionValue* args) {
    (void)argc; (void)args;
    return make_str("0.1.0");
}

static LionModuleFunc funcs[] = {
    {"arange",    panda_arange},
    {"zeros",     panda_zeros},
    {"ones",      panda_ones},
    {"linspace",  panda_linspace},
    {"sum",       panda_sum},
    {"mean",      panda_mean},
    {"min",       panda_min},
    {"max",       panda_max},
    {"abs",       panda_abs},
    {"sin",       panda_sin},
    {"cos",       panda_cos},
    {"sqrt",      panda_sqrt},
    {"pow",       panda_pow},
    {"add",       panda_add},
    {"sub",       panda_sub},
    {"mul",       panda_mul},
    {"dot",       panda_dot},
    {"std",       panda_std},
    {"shape",     panda_shape},
    {"reshape",   panda_reshape},
    {"eye",       panda_eye},
    {"version",   panda_version},
};

int lion_module_init(int* out_count, LionModuleFunc** out_funcs) {
    *out_count = sizeof(funcs) / sizeof(funcs[0]);
    *out_funcs = funcs;
    return 0;
}
