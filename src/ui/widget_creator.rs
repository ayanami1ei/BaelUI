use anyhow::Result;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ui::widget::Widget;

type WidgetHandle = Arc<Mutex<Box<dyn Widget + Send + Sync>>>;

/// Creator: 管理 widget 的创建与注册，维护下一个 id 和 id->handle 映射
pub struct Creator {
    next_id: u64,
    registry: HashMap<u64, WidgetHandle>,
}

impl Creator {
    pub fn new() -> Self {
        Creator {
            next_id: 1,
            registry: HashMap::new(),
        }
    }

    /// 创建并注册一个新 widget，返回分配的 id
    pub fn create<T: Widget + Send + Sync + 'static>(
        &mut self,
        window_id: winit::window::WindowId,
        pattern: &Vec<crate::render::vertex::Vertex>,
        texture: &Vec<u8>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        // Allocate uninitialized memory for T and initialize it in-place by
        // calling the instance method `new(&mut self, ...)` on that memory.
        // This uses unsafe and requires the implementor's `new` to fully
        // initialize all fields before any reads occur.
        let boxed_uninit = Box::new(std::mem::MaybeUninit::<T>::uninit());
        let raw: *mut T = Box::into_raw(boxed_uninit) as *mut T;
        unsafe {
            (&mut *raw).new(id, window_id, pattern, texture);
            let w: Box<T> = Box::from_raw(raw);
            let obj: Box<dyn Widget + Send + Sync> = w;
            let handle: WidgetHandle = Arc::new(Mutex::new(obj));
            self.registry.insert(id, handle);
        }
        id
    }

    /// 注册已经构造好的 widget
    pub fn register(&mut self, widget_id: u64, widget: Box<dyn Widget + Send + Sync>) -> WidgetHandle {
        let handle: WidgetHandle = Arc::new(Mutex::new(widget));
        self.registry.insert(widget_id, handle.clone());
        handle
    }

    /// 根据 id 获取 handle
    pub fn get_handle(&self, id: u64) -> Option<WidgetHandle> {
        self.registry.get(&id).map(|h| h.clone())
    }

    /// 将 child_id 添加为 parent_id 的子项（会调用父的 `add` 与子 `set_father_widget`）
    pub fn add_child(&mut self, parent_id: u64, child_id: u64) -> Result<()> {
        if let Some(ph) = self.registry.get(&parent_id) {
            ph.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock parent widget: {}", e))?
                .add(child_id);
        }
        if let Some(ch) = self.registry.get(&child_id) {
            ch.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock child widget: {}", e))?
                .set_father_widget(parent_id);
        }

        Ok(())
    }

    /// 设定父项（调用子对象与父对象的接口）
    pub fn set_father(&mut self, child_id: u64, father_id: u64) -> Result<()> {
        if let Some(ch) = self.registry.get(&child_id) {
            ch.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock child widget: {}", e))?
                .set_father_widget(father_id);
        }
        if let Some(ph) = self.registry.get(&father_id) {
            ph.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock parent widget: {}", e))?
                .add(child_id);
        }
        Ok(())
    }

    /// 获取子项集合（id -> handle）
    pub fn get_sons(&self, id: u64) -> HashMap<u64, WidgetHandle> {
        if let Some(h) = self.get_handle(id) {
            h.lock().map(|h| h.get_sons()).unwrap_or_else(|e| {
                eprintln!("Failed to lock widget {}: {}", id, e);
                HashMap::new()
            })
        } else {
            HashMap::new()
        }
    }

    /// 获取父项的 handle（如果存在）
    pub fn get_father(&self, id: u64) -> Option<WidgetHandle> {
        self.get_handle(id).and_then(|h| h.lock().map(|h| h.get_father()).unwrap_or_else(|e| {
            eprintln!("Failed to lock widget {}: {}", id, e);
            None
        }))
    }
}

/// 全局单例 Creator，方便直接调用 `crate::ui::widget_creator::create::<T>(...)`
pub static GLOBAL_CREATOR: Lazy<Mutex<Creator>> = Lazy::new(|| Mutex::new(Creator::new()));

/// 全局快捷函数：在全局 Creator 上创建一个 widget，返回 id
pub fn create<T: Widget + Send + Sync + 'static>(
    window_id: winit::window::WindowId,
    pattern: &Vec<crate::render::vertex::Vertex>,
    texture: &Vec<u8>,
) -> u64 {
    let mut g = GLOBAL_CREATOR.lock().unwrap();
    g.create::<T>(window_id, pattern, texture)
}

pub fn register(widget_id: u64, widget: Box<dyn Widget + Send + Sync>) -> WidgetHandle {
    let mut g = GLOBAL_CREATOR.lock().unwrap();
    g.register(widget_id, widget)
}

pub fn get_handle(id: u64) -> Option<WidgetHandle> {
    let g = GLOBAL_CREATOR.lock().unwrap();
    g.get_handle(id)
}

pub fn add_child(parent_id: u64, child_id: u64) -> Result<()> {
    let mut g = GLOBAL_CREATOR.lock().unwrap();
    g.add_child(parent_id, child_id)
}

pub fn set_father(child_id: u64, father_id: u64) -> Result<()> {
    let mut g = GLOBAL_CREATOR.lock().unwrap();
    g.set_father(child_id, father_id)
}

pub fn get_sons(id: u64) -> HashMap<u64, WidgetHandle> {
    let g = GLOBAL_CREATOR.lock().unwrap();
    g.get_sons(id)
}

pub fn get_father(id: u64) -> Option<WidgetHandle> {
    let g = GLOBAL_CREATOR.lock().unwrap();
    g.get_father(id)
}
