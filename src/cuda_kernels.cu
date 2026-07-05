extern "C" __global__ void vec_add(double *a, double *b, double *c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) c[idx] = a[idx] + b[idx];
}

extern "C" __global__ void vec_sub(double *a, double *b, double *c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) c[idx] = a[idx] - b[idx];
}

extern "C" __global__ void vec_mul(double *a, double *b, double *c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) c[idx] = a[idx] * b[idx];
}

extern "C" __global__ void scalar_mul(double *a, double s, double *c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) c[idx] = a[idx] * s;
}

extern "C" __global__ void relu_activation(double *a, double *c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) c[idx] = a[idx] > 0.0 ? a[idx] : 0.0;
}

extern "C" __global__ void sigmoid_activation(double *a, double *c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) c[idx] = 1.0 / (1.0 + exp(-a[idx]));
}

extern "C" __global__ void tanh_activation(double *a, double *c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) c[idx] = tanh(a[idx]);
}

extern "C" __global__ void transpose(double *src, double *dst, int rows, int cols) {
    int r = blockIdx.y * blockDim.y + threadIdx.y;
    int c = blockIdx.x * blockDim.x + threadIdx.x;
    if (r < rows && c < cols) {
        dst[c * rows + r] = src[r * cols + c];
    }
}
