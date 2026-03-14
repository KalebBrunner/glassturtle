use crate::init::State;

pub fn update_state(state: &mut State<'_>) {
    match state.render() {
        Ok(_) => {}
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            state.update_surface();
            state.resize(state.size);
        }
        Err(e) => eprintln!("{:?}", e),
    }
}
