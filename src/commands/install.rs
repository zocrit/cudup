use crate::cuda::discover;

pub fn install(version: &Option<String>) {
    discover::fetch_available_cuda_versions();
    println!("Available CUDA versions: {:?}", ());
}
