
use std::sync::OnceLock;

// ── CUDA Runtime API types ──

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cudaDeviceProp {
    pub name: [i8; 256],
    pub totalGlobalMem: usize,
    pub sharedMemPerBlock: usize,
    pub regsPerBlock: i32,
    pub warpSize: i32,
    pub memPitch: usize,
    pub maxThreadsPerBlock: i32,
    pub maxThreadsDim: [i32; 3],
    pub maxGridSize: [i32; 3],
    pub clockRate: i32,
    pub totalConstMem: usize,
    pub major: i32,
    pub minor: i32,
    pub textureAlignment: usize,
    pub texturePitchAlignment: usize,
    pub deviceOverlap: i32,
    pub multiProcessorCount: i32,
    pub kernelExecTimeoutEnabled: i32,
    pub integrated: i32,
    pub canMapHostMemory: i32,
    pub computeMode: i32,
    pub maxTexture1D: i32,
    pub maxTexture1DMipmap: i32,
    pub maxTexture1DLinear: i32,
    pub maxTexture2D: [i32; 2],
    pub maxTexture2DMipmap: [i32; 2],
    pub maxTexture2DLinear: [i32; 3],
    pub maxTexture2DGather: [i32; 2],
    pub maxTexture3D: [i32; 3],
    pub maxTexture3DAlt: [i32; 3],
    pub maxTextureCubemap: i32,
    pub maxTexture1DLayered: [i32; 2],
    pub maxTexture2DLayered: [i32; 3],
    pub maxTextureCubemapLayered: [i32; 2],
    pub maxSurface1D: i32,
    pub maxSurface2D: [i32; 2],
    pub maxSurface3D: [i32; 3],
    pub maxSurface1DLayered: [i32; 2],
    pub maxSurface2DLayered: [i32; 3],
    pub maxSurfaceCubemap: i32,
    pub maxSurfaceCubemapLayered: [i32; 2],
    pub surfaceAlignment: usize,
    pub concurrentKernels: i32,
    pub ECCEnabled: i32,
    pub pciBusID: i32,
    pub pciDeviceID: i32,
    pub pciDomainID: i32,
    pub tccDriver: i32,
    pub asyncEngineCount: i32,
    pub unifiedAddressing: i32,
    pub memoryClockRate: i32,
    pub memoryBusWidth: i32,
    pub l2CacheSize: i32,
    pub persistentL2CacheMaxSize: i32,
    pub maxThreadsPerMultiProcessor: i32,
    pub accessPolicyMaxWindowSize: i32,
    pub reservedSharedMemPerBlock: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cublasContext {
    _private: [u8; 0],
}

pub type cudaError_t = i32;
pub type cublasStatus_t = i32;
pub type cublasHandle_t = *mut cublasContext;
pub type CUmodule = *mut std::ffi::c_void;
pub type CUfunction = *mut std::ffi::c_void;
pub type CUstream = *mut std::ffi::c_void;

pub const cudaSuccess: cudaError_t = 0;
pub const cudaErrorMemoryAllocation: cudaError_t = 2;
pub const CUBLAS_STATUS_SUCCESS: cublasStatus_t = 0;

// ── CUDA Runtime API FFI ──

extern "C" {
    fn cudaSetDevice(device: i32) -> cudaError_t;
    fn cudaGetDeviceCount(count: *mut i32) -> cudaError_t;
    fn cudaGetDeviceProperties(prop: *mut cudaDeviceProp, device: i32) -> cudaError_t;
    fn cudaMalloc(devPtr: *mut *mut std::ffi::c_void, size: usize) -> cudaError_t;
    fn cudaFree(devPtr: *mut std::ffi::c_void) -> cudaError_t;
    fn cudaMemcpy(dst: *mut std::ffi::c_void, src: *const std::ffi::c_void, count: usize, kind: i32) -> cudaError_t;
    fn cudaGetLastError() -> cudaError_t;
}

pub const cudaMemcpyHostToDevice: i32 = 1;
pub const cudaMemcpyDeviceToHost: i32 = 2;
pub const cudaMemcpyDeviceToDevice: i32 = 3;

// ── CUDA Driver API FFI ──

extern "C" {
    fn cuModuleLoadData(module: *mut CUmodule, image: *const std::ffi::c_void) -> cudaError_t;
    fn cuModuleGetFunction(func: *mut CUfunction, module: CUmodule, name: *const std::ffi::c_void) -> cudaError_t;
    fn cuLaunchKernel(
        func: CUfunction,
        gridDimX: u32, gridDimY: u32, gridDimZ: u32,
        blockDimX: u32, blockDimY: u32, blockDimZ: u32,
        sharedMemBytes: u32,
        stream: CUstream,
        kernelParams: *mut *mut std::ffi::c_void,
        extra: *mut std::ffi::c_void,
    ) -> cudaError_t;
    fn cuCtxSynchronize() -> cudaError_t;
}

// ── cuBLAS API FFI ──

extern "C" {
    fn cublasCreate_v2(handle: *mut cublasHandle_t) -> cublasStatus_t;
    fn cublasDestroy_v2(handle: cublasHandle_t) -> cublasStatus_t;
    fn cublasDgemm_v2(
        handle: cublasHandle_t,
        transa: i32, transb: i32,
        m: i32, n: i32, k: i32,
        alpha: *const f64,
        A: *const f64, lda: i32,
        B: *const f64, ldb: i32,
        beta: *const f64,
        C: *mut f64, ldc: i32,
    ) -> cublasStatus_t;
    fn cublasSgemm_v2(
        handle: cublasHandle_t,
        transa: i32, transb: i32,
        m: i32, n: i32, k: i32,
        alpha: *const f32,
        A: *const f32, lda: i32,
        B: *const f32, ldb: i32,
        beta: *const f32,
        C: *mut f32, ldc: i32,
    ) -> cublasStatus_t;
}

pub const CUBLAS_OP_N: i32 = 0;
pub const CUBLAS_OP_T: i32 = 1;

// ── Global state ──

struct CudaState {
    module: CUmodule,
    blas: cublasHandle_t,
}

unsafe impl Send for CudaState {}
unsafe impl Sync for CudaState {}

static STATE: OnceLock<CudaState> = OnceLock::new();

fn get_state() -> &'static CudaState {
    STATE.get().expect("CUDA not initialized")
}

pub fn is_initialized() -> bool {
    STATE.get().is_some()
}

pub fn init() -> Result<(), String> {
    let mut count: i32 = 0;
    unsafe {
        check_runtime(cudaGetDeviceCount(&mut count))?;
        if count == 0 { return Err("no CUDA devices found".to_string()); }
        check_runtime(cudaSetDevice(0))?;

        // Load PTX module
        let ptx_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/cuda_kernels.ptx"));
        let mut module: CUmodule = std::ptr::null_mut();
        check_runtime(cuModuleLoadData(
            &mut module,
            ptx_bytes.as_ptr() as *const std::ffi::c_void,
        ))?;

        // Create cuBLAS handle
        let mut blas: cublasHandle_t = std::ptr::null_mut();
        check_blas(cublasCreate_v2(&mut blas))?;

        STATE.set(CudaState { module, blas }).map_err(|_| "CUDA already initialized".to_string())?;
    }
    Ok(())
}

pub fn device_count() -> Result<i32, String> {
    let mut count: i32 = 0;
    unsafe { check_runtime(cudaGetDeviceCount(&mut count))?; }
    Ok(count)
}

// ── Memory management ──

pub fn gpu_alloc(size: usize) -> Result<*mut std::ffi::c_void, String> {
    let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    unsafe { check_runtime(cudaMalloc(&mut ptr, size))?; }
    Ok(ptr)
}

pub fn gpu_free(ptr: *mut std::ffi::c_void) -> Result<(), String> {
    if !ptr.is_null() {
        unsafe { check_runtime(cudaFree(ptr))?; }
    }
    Ok(())
}

pub fn copy_to_gpu(dst: *mut std::ffi::c_void, src: *const std::ffi::c_void, size: usize) -> Result<(), String> {
    unsafe { check_runtime(cudaMemcpy(dst, src, size, cudaMemcpyHostToDevice))?; }
    Ok(())
}

pub fn copy_to_cpu(dst: *mut std::ffi::c_void, src: *const std::ffi::c_void, size: usize) -> Result<(), String> {
    unsafe { check_runtime(cudaMemcpy(dst, src, size, cudaMemcpyDeviceToHost))?; }
    Ok(())
}

pub fn copy_gpu_to_gpu(dst: *mut std::ffi::c_void, src: *const std::ffi::c_void, size: usize) -> Result<(), String> {
    unsafe { check_runtime(cudaMemcpy(dst, src, size, cudaMemcpyDeviceToDevice))?; }
    Ok(())
}

pub fn sync() -> Result<(), String> {
    unsafe { check_runtime(cuCtxSynchronize())?; }
    Ok(())
}

// ── Kernel launching ──

pub fn get_kernel(name: &str) -> Result<CUfunction, String> {
    let state = get_state();
    let mut func: CUfunction = std::ptr::null_mut();
    let cname = std::ffi::CString::new(name).map_err(|e| e.to_string())?;
    unsafe {
        check_runtime(cuModuleGetFunction(
            &mut func,
            state.module,
            cname.as_ptr() as *const std::ffi::c_void,
        ))?;
    }
    Ok(func)
}

pub fn launch_1d(func: CUfunction, n: usize, args: &[*mut std::ffi::c_void]) -> Result<(), String> {
    let block = 256u32;
    let grid = ((n as u32 + block - 1) / block).max(1);
    unsafe {
        check_runtime(cuLaunchKernel(
            func, grid, 1, 1, block, 1, 1, 0, std::ptr::null_mut(),
            args.as_ptr() as *mut *mut std::ffi::c_void,
            std::ptr::null_mut(),
        ))?;
    }
    Ok(())
}

pub fn launch_2d(func: CUfunction, width: u32, height: u32, args: &[*mut std::ffi::c_void]) -> Result<(), String> {
    let bx = 16u32;
    let by = 16u32;
    let gx = (width + bx - 1) / bx;
    let gy = (height + by - 1) / by;
    unsafe {
        check_runtime(cuLaunchKernel(
            func, gx, gy, 1, bx, by, 1, 0, std::ptr::null_mut(),
            args.as_ptr() as *mut *mut std::ffi::c_void,
            std::ptr::null_mut(),
        ))?;
    }
    Ok(())
}

// ── cuBLAS wrappers ──

pub fn blas_dgemm(
    transa: bool, transb: bool,
    m: i32, n: i32, k: i32,
    alpha: f64,
    a: *const f64, lda: i32,
    b: *const f64, ldb: i32,
    beta: f64,
    c: *mut f64, ldc: i32,
) -> Result<(), String> {
    let state = get_state();
    let ta = if transa { CUBLAS_OP_T } else { CUBLAS_OP_N };
    let tb = if transb { CUBLAS_OP_T } else { CUBLAS_OP_N };
    unsafe {
        check_blas(cublasDgemm_v2(
            state.blas, ta, tb, m, n, k,
            &alpha, a, lda, b, ldb, &beta, c, ldc,
        ))?;
    }
    Ok(())
}

// ── Error checking ──

fn check_runtime(e: cudaError_t) -> Result<(), String> {
    if e != cudaSuccess {
        Err(format!("CUDA error: {}", e))
    } else {
        Ok(())
    }
}

fn check_blas(e: cublasStatus_t) -> Result<(), String> {
    if e != CUBLAS_STATUS_SUCCESS {
        Err(format!("cuBLAS error: {}", e))
    } else {
        Ok(())
    }
}
