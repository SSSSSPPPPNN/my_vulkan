#[derive(Debug)]
///命令，用于主线程给渲染线程发指令
pub enum RenderCommand {
    Resized { width: u32, height: u32 },
    Exit,
}
