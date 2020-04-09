use crate::renderer::Renderer;

trait Presenter {
    fn present(renderer: impl Renderer);
}
