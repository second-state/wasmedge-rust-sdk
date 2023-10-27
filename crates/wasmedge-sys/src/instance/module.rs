//! Defines WasmEdge Instance and other relevant types.
use crate::{
    ffi::{self},
    instance::{global::InnerGlobal, memory::InnerMemory, table::InnerTable},
    types::WasmEdgeString,
    FuncRef, Function, Global, Memory, Table, WasmEdgeResult,
};

use wasmedge_types::error::{InstanceError, WasmEdgeError};

use super::{function::AsFunc, InnerRef};

/// An [Instance] represents an instantiated module. In the instantiation process, An [Instance] is created from al[Module](crate::Module). From an [Instance] the exported [functions](crate::Function), [tables](crate::Table), [memories](crate::Memory), and [globals](crate::Global) can be fetched.
#[derive(Debug)]
pub struct Instance {
    pub(crate) inner: InnerInstance,
}
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            ffi::WasmEdge_ModuleInstanceDelete(self.inner.0);
        }
    }
}
impl AsInstance for Instance {
    unsafe fn as_ptr(&self) -> *const ffi::WasmEdge_ModuleInstanceContext {
        self.inner.0
    }
}

impl<Inst: Sized> AsInstance for Inst
where
    Inst: AsMut<Instance> + AsRef<Instance>,
{
    unsafe fn as_ptr(&self) -> *const ffi::WasmEdge_ModuleInstanceContext {
        self.as_ref().as_ptr()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InnerInstance(pub(crate) *mut ffi::WasmEdge_ModuleInstanceContext);
unsafe impl Send for InnerInstance {}
unsafe impl Sync for InnerInstance {}

/// The object as an module instance is required to implement this trait.
pub trait AsInstance {
    /// Returns the name of this exported [module instance](crate::Instance).
    ///
    /// If this module instance is an active module instance, then None is returned.
    fn name(&self) -> Option<String> {
        let name = unsafe { ffi::WasmEdge_ModuleInstanceGetModuleName(self.as_ptr()) };

        let name: String = (&name).into();
        if name.is_empty() {
            return None;
        }

        Some(name)
    }

    /// Returns the exported [table instance](crate::Table) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [table instance](crate::Table).
    ///
    /// # Error
    ///
    /// If fail to find the target [table instance](crate::Table), then an error is returned.
    fn get_table(&self, name: impl AsRef<str>) -> WasmEdgeResult<InnerRef<Table, &Self>>
    where
        Self: Sized,
    {
        let table_name: WasmEdgeString = name.as_ref().into();
        let ctx =
            unsafe { ffi::WasmEdge_ModuleInstanceFindTable(self.as_ptr(), table_name.as_raw()) };
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Instance(
                InstanceError::NotFoundTable(name.as_ref().to_string()),
            ))),
            false => {
                let table = std::mem::ManuallyDrop::new(Table {
                    inner: InnerTable(ctx),
                });
                Ok(unsafe { InnerRef::create_from_ref(table, self) })
            }
        }
    }

    /// Returns the exported [memory instance](crate::Memory) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [memory instance](crate::Memory).
    ///
    /// # Error
    ///
    /// If fail to find the target [memory instance](crate::Memory), then an error is returned.
    fn get_memory_ref(&self, name: impl AsRef<str>) -> WasmEdgeResult<InnerRef<Memory, &Self>>
    where
        Self: Sized,
    {
        unsafe {
            let mem_name: WasmEdgeString = name.as_ref().into();
            let ctx = ffi::WasmEdge_ModuleInstanceFindMemory(self.as_ptr(), mem_name.as_raw());

            if ctx.is_null() {
                Err(Box::new(WasmEdgeError::Instance(
                    InstanceError::NotFoundMem(name.as_ref().to_string()),
                )))
            } else {
                let mem = Memory {
                    inner: InnerMemory(ctx),
                };

                Ok(InnerRef::create_from_ref(
                    std::mem::ManuallyDrop::new(mem),
                    self,
                ))
            }
        }
    }

    fn get_memory_mut(
        &mut self,
        name: impl AsRef<str>,
    ) -> WasmEdgeResult<InnerRef<Memory, &mut Self>>
    where
        Self: Sized,
    {
        unsafe {
            let mem_name: WasmEdgeString = name.as_ref().into();
            let ctx = ffi::WasmEdge_ModuleInstanceFindMemory(self.as_ptr(), mem_name.as_raw());

            if ctx.is_null() {
                Err(Box::new(WasmEdgeError::Instance(
                    InstanceError::NotFoundMem(name.as_ref().to_string()),
                )))
            } else {
                let mem = Memory {
                    inner: InnerMemory(ctx),
                };

                Ok(InnerRef::create_from_mut(
                    std::mem::ManuallyDrop::new(mem),
                    self,
                ))
            }
        }
    }

    /// Returns the exported [global instance](crate::Global) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [global instance](crate::Global).
    ///
    /// # Error
    ///
    /// If fail to find the target [global instance](crate::Global), then an error is returned.
    fn get_global(&self, name: impl AsRef<str>) -> WasmEdgeResult<InnerRef<Global, &Self>>
    where
        Self: Sized,
    {
        let global_name: WasmEdgeString = name.as_ref().into();
        let ctx =
            unsafe { ffi::WasmEdge_ModuleInstanceFindGlobal(self.as_ptr(), global_name.as_raw()) };
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Instance(
                InstanceError::NotFoundGlobal(name.as_ref().to_string()),
            ))),
            false => {
                let value = std::mem::ManuallyDrop::new(Global {
                    inner: InnerGlobal(ctx),
                });
                Ok(unsafe { InnerRef::create_from_ref(value, self) })
            }
        }
    }

    fn get_global_mut(
        &mut self,
        name: impl AsRef<str>,
    ) -> WasmEdgeResult<InnerRef<Global, &mut Self>>
    where
        Self: Sized,
    {
        let global_name: WasmEdgeString = name.as_ref().into();
        let ctx =
            unsafe { ffi::WasmEdge_ModuleInstanceFindGlobal(self.as_ptr(), global_name.as_raw()) };
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Instance(
                InstanceError::NotFoundGlobal(name.as_ref().to_string()),
            ))),
            false => {
                let value = std::mem::ManuallyDrop::new(Global {
                    inner: InnerGlobal(ctx),
                });
                Ok(unsafe { InnerRef::create_from_mut(value, self) })
            }
        }
    }

    /// Returns the length of the exported [function instances](crate::Function) in this module instance.
    fn func_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListFunctionLength(self.as_ptr()) }
    }

    /// Returns the names of the exported [function instances](crate::Function) in this module instance.
    fn func_names(&self) -> Option<Vec<String>> {
        let len_func_names = self.func_len();
        match len_func_names > 0 {
            true => {
                let mut func_names = Vec::with_capacity(len_func_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListFunction(
                        self.as_ptr(),
                        func_names.as_mut_ptr(),
                        len_func_names,
                    );
                    func_names.set_len(len_func_names as usize);
                }

                let names = func_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }

    /// Returns the exported [function instance](crate::Function) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [function instance](crate::Function).
    ///
    /// # Error
    ///
    /// If fail to find the target [function](crate::Function), then an error is returned.
    fn get_func(&self, name: &str) -> WasmEdgeResult<FuncRef<&Instance>> {
        unsafe {
            let func_name: WasmEdgeString = name.into();
            let func_ctx =
                ffi::WasmEdge_ModuleInstanceFindFunction(self.as_ptr(), func_name.as_raw());

            if func_ctx.is_null() {
                Err(Box::new(WasmEdgeError::Instance(
                    InstanceError::NotFoundFunc(name.to_string()),
                )))
            } else {
                let value = Function::from_raw(func_ctx);
                Ok(FuncRef::create_ref(std::mem::ManuallyDrop::new(value)))
            }
        }
    }

    /// Returns the exported [function instance](crate::Function) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [function instance](crate::Function).
    ///
    /// # Error
    ///
    /// If fail to find the target [function](crate::Function), then an error is returned.
    fn get_func_mut(&mut self, name: &str) -> WasmEdgeResult<FuncRef<&mut Instance>> {
        unsafe {
            let func_name: WasmEdgeString = name.into();
            let func_ctx =
                ffi::WasmEdge_ModuleInstanceFindFunction(self.as_ptr(), func_name.as_raw());

            if func_ctx.is_null() {
                Err(Box::new(WasmEdgeError::Instance(
                    InstanceError::NotFoundFunc(name.to_string()),
                )))
            } else {
                let value = Function::from_raw(func_ctx);
                Ok(FuncRef::create_mut(std::mem::ManuallyDrop::new(value)))
            }
        }
    }

    /// Returns the length of the exported [table instances](crate::Table) in this module instance.
    fn table_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListTableLength(self.as_ptr()) }
    }

    /// Returns the names of the exported [table instances](crate::Table) in this module instance.
    fn table_names(&self) -> Option<Vec<String>> {
        let len_table_names = self.table_len();
        match len_table_names > 0 {
            true => {
                let mut table_names = Vec::with_capacity(len_table_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListTable(
                        self.as_ptr(),
                        table_names.as_mut_ptr(),
                        len_table_names,
                    );
                    table_names.set_len(len_table_names as usize);
                }

                let names = table_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }

    /// Returns the length of the exported [memory instances](crate::Memory) in this module instance.
    fn mem_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListMemoryLength(self.as_ptr()) }
    }

    /// Returns the names of all exported [memory instances](crate::Memory) in this module instance.
    fn mem_names(&self) -> Option<Vec<String>> {
        let len_mem_names = self.mem_len();
        match len_mem_names > 0 {
            true => {
                let mut mem_names = Vec::with_capacity(len_mem_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListMemory(
                        self.as_ptr(),
                        mem_names.as_mut_ptr(),
                        len_mem_names,
                    );
                    mem_names.set_len(len_mem_names as usize);
                }

                let names = mem_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }

    /// Returns the length of the exported [global instances](crate::Global) in this module instance.
    fn global_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListGlobalLength(self.as_ptr()) }
    }

    /// Returns the names of the exported [global instances](crate::Global) in this module instance.
    fn global_names(&self) -> Option<Vec<String>> {
        let len_global_names = self.global_len();
        match len_global_names > 0 {
            true => {
                let mut global_names = Vec::with_capacity(len_global_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListGlobal(
                        self.as_ptr(),
                        global_names.as_mut_ptr(),
                        len_global_names,
                    );
                    global_names.set_len(len_global_names as usize);
                }

                let names = global_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }

    /// # Safety
    ///
    /// Provides a raw pointer to the inner module instance context.
    /// The lifetime of the returned pointer must not exceed that of the object itself.
    unsafe fn as_ptr(&self) -> *const ffi::WasmEdge_ModuleInstanceContext;
}

/// An [ImportModule] represents a host module with a name. A host module consists of one or more host [function](crate::Function), [table](crate::Table), [memory](crate::Memory), and [global](crate::Global) instances,  which are defined outside wasm modules and fed into wasm modules as imports.
#[derive(Debug)]
pub struct ImportModule<T: ?Sized> {
    pub(crate) inner: InnerInstance,
    name: String,
    _data: std::marker::PhantomData<T>,
}
impl<T: ?Sized> Drop for ImportModule<T> {
    fn drop(&mut self) {
        unsafe {
            ffi::WasmEdge_ModuleInstanceDelete(self.inner.0);
        }
    }
}

unsafe extern "C" fn import_data_finalizer<T>(ptr: *mut std::os::raw::c_void) {
    let box_data: Box<T> = Box::from_raw(ptr as _);
    std::mem::drop(box_data)
}

impl<T: Sized> ImportModule<T> {
    /// Creates a module instance which is used to import host functions, tables, memories, and globals into a wasm module.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the import module instance.
    ///
    /// * `data` - The host context data used in this function.
    ///
    /// # Error
    ///
    /// If fail to create the import module instance, then an error is returned.
    pub fn create(name: impl AsRef<str>, data: Box<T>) -> WasmEdgeResult<Self> {
        let raw_name = WasmEdgeString::from(name.as_ref());

        // ffi::WasmEdge_ModuleInstanceGetModuleName(Cxt)

        let ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceCreateWithData(
                raw_name.as_raw(),
                Box::leak(data) as *mut _ as *mut std::ffi::c_void,
                Some(import_data_finalizer::<T>),
            )
        };

        let import = Self {
            inner: InnerInstance(ctx),
            name: name.as_ref().to_string(),
            _data: Default::default(),
        };

        Ok(import)
    }

    /// # Safety
    ///
    /// Provides a raw pointer to the inner module instance context.
    /// The lifetime of the returned pointer must not exceed that of the object itself.
    pub unsafe fn as_raw(&self) -> *mut ffi::WasmEdge_ModuleInstanceContext {
        self.inner.0
    }

    /// # Safety
    ///
    /// This function will take over the lifetime management of `ctx`, so do not call `ffi::WasmEdge_ModuleInstanceDelete` on `ctx` after this.
    pub unsafe fn from_raw(ctx: *mut ffi::WasmEdge_ModuleInstanceContext) -> Self {
        let wasmedge_s = WasmEdgeString::from_raw(ffi::WasmEdge_ModuleInstanceGetModuleName(ctx));
        let name = (&wasmedge_s).into();
        Self {
            inner: InnerInstance(ctx),
            name,
            _data: Default::default(),
        }
    }

    pub fn get_host_data(&self) -> &T {
        unsafe { &*(ffi::WasmEdge_ModuleInstanceGetHostData(self.as_ptr()) as *mut T) }
    }

    pub fn get_host_data_mut(&mut self) -> &mut T {
        unsafe { &mut *(ffi::WasmEdge_ModuleInstanceGetHostData(self.as_ptr()) as *mut T) }
    }
}
impl<T: Sized> AsInstance for ImportModule<T> {
    unsafe fn as_ptr(&self) -> *const ffi::WasmEdge_ModuleInstanceContext {
        self.inner.0
    }

    fn name(&self) -> Option<String> {
        Some(self.name.clone())
    }
}
impl<T: Sized> ImportModule<T> {
    pub fn add_func(&mut self, name: impl AsRef<str>, func: Function) {
        let func_name: WasmEdgeString = name.into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddFunction(
                self.inner.0,
                func_name.as_raw(),
                func.get_func_raw(),
            );
        }
        std::mem::forget(func);
    }

    pub fn add_table(&mut self, name: impl AsRef<str>, table: Table) {
        let table_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddTable(self.inner.0, table_name.as_raw(), table.inner.0);
        }
        std::mem::forget(table);
    }

    pub fn add_memory(&mut self, name: impl AsRef<str>, memory: Memory) {
        let mem_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddMemory(self.inner.0, mem_name.as_raw(), memory.inner.0);
        }
        std::mem::forget(memory);
    }

    pub fn add_global(&mut self, name: impl AsRef<str>, global: Global) {
        let global_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddGlobal(
                self.inner.0,
                global_name.as_raw(),
                global.inner.0,
            );
        }
        std::mem::forget(global);
    }
}

/// A [WasiModule] is a module instance for the WASI specification.
#[derive(Debug, Clone)]
pub struct WasiModule {
    pub(crate) inner: InnerInstance,
}

impl Drop for WasiModule {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_ModuleInstanceDelete(self.inner.0) };
    }
}
impl WasiModule {
    /// Creates a WASI host module which contains the WASI host functions, and initializes it with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables in the format `ENV_VAR_NAME=VALUE`.
    ///
    /// * `preopens` - The directories to pre-open. The required format is `GUEST_PATH:HOST_PATH`.
    ///
    /// # Error
    ///
    /// If fail to create a host module, then an error is returned.
    pub fn create(
        args: Option<Vec<&str>>,
        envs: Option<Vec<&str>>,
        preopens: Option<Vec<&str>>,
    ) -> WasmEdgeResult<Self> {
        // parse args
        let cstr_args: Vec<_> = match args {
            Some(args) => args
                .iter()
                .map(|&x| std::ffi::CString::new(x).unwrap())
                .collect(),
            None => vec![],
        };
        let mut p_args: Vec<_> = cstr_args.iter().map(|x| x.as_ptr()).collect();
        let p_args_len = p_args.len();
        p_args.push(std::ptr::null());

        // parse envs
        let cstr_envs: Vec<_> = match envs {
            Some(envs) => envs
                .iter()
                .map(|&x| std::ffi::CString::new(x).unwrap())
                .collect(),
            None => vec![],
        };
        let mut p_envs: Vec<_> = cstr_envs.iter().map(|x| x.as_ptr()).collect();
        let p_envs_len = p_envs.len();
        p_envs.push(std::ptr::null());

        // parse preopens
        let cstr_preopens: Vec<_> = match preopens {
            Some(preopens) => preopens
                .iter()
                .map(|&x| std::ffi::CString::new(x).unwrap())
                .collect(),
            None => vec![],
        };
        let mut p_preopens: Vec<_> = cstr_preopens.iter().map(|x| x.as_ptr()).collect();
        let p_preopens_len = p_preopens.len();
        p_preopens.push(std::ptr::null());

        let ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceCreateWASI(
                p_args.as_ptr(),
                p_args_len as u32,
                p_envs.as_ptr(),
                p_envs_len as u32,
                p_preopens.as_ptr(),
                p_preopens_len as u32,
            )
        };
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::ImportObjCreate)),
            false => Ok(Self {
                inner: InnerInstance(ctx),
            }),
        }
    }

    /// Returns the name of the module instance.
    pub fn name(&self) -> &str {
        "wasi_snapshot_preview1"
    }

    /// Initializes the WASI host module with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables in the format `ENV_VAR_NAME=VALUE`.
    ///
    /// * `preopens` - The directories to pre-open. The required format is `GUEST_PATH:HOST_PATH`.
    pub fn init_wasi(
        &mut self,
        args: Option<Vec<&str>>,
        envs: Option<Vec<&str>>,
        preopens: Option<Vec<&str>>,
    ) {
        // parse args
        let cstr_args: Vec<_> = match args {
            Some(args) => args
                .iter()
                .map(|&x| std::ffi::CString::new(x).unwrap())
                .collect(),
            None => vec![],
        };
        let mut p_args: Vec<_> = cstr_args.iter().map(|x| x.as_ptr()).collect();
        let p_args_len = p_args.len();
        p_args.push(std::ptr::null());

        // parse envs
        let cstr_envs: Vec<_> = match envs {
            Some(envs) => envs
                .iter()
                .map(|&x| std::ffi::CString::new(x).unwrap())
                .collect(),
            None => vec![],
        };
        let mut p_envs: Vec<_> = cstr_envs.iter().map(|x| x.as_ptr()).collect();
        let p_envs_len = p_envs.len();
        p_envs.push(std::ptr::null());

        // parse preopens
        let cstr_preopens: Vec<_> = match preopens {
            Some(preopens) => preopens
                .iter()
                .map(|&x| std::ffi::CString::new(x).unwrap())
                .collect(),
            None => vec![],
        };
        let mut p_preopens: Vec<_> = cstr_preopens.iter().map(|x| x.as_ptr()).collect();
        let p_preopens_len = p_preopens.len();
        p_preopens.push(std::ptr::null());

        unsafe {
            ffi::WasmEdge_ModuleInstanceInitWASI(
                self.inner.0,
                p_args.as_ptr(),
                p_args_len as u32,
                p_envs.as_ptr(),
                p_envs_len as u32,
                p_preopens.as_ptr(),
                p_preopens_len as u32,
            )
        };
    }

    /// Returns the WASI exit code.
    ///
    /// The WASI exit code can be accessed after running the "_start" function of a `wasm32-wasi` program.
    pub fn exit_code(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceWASIGetExitCode(self.inner.0 as *const _) }
    }

    /// Returns the native handler from the mapped FD/Handler.
    ///
    /// # Argument
    ///
    /// * `fd` - The WASI mapped Fd.
    ///
    /// # Error
    ///
    /// If fail to get the native handler, then an error is returned.
    pub fn get_native_handler(&self, fd: i32) -> WasmEdgeResult<u64> {
        let mut handler: u64 = 0;
        let code: u32 = unsafe {
            ffi::WasmEdge_ModuleInstanceWASIGetNativeHandler(
                self.inner.0 as *const _,
                fd,
                &mut handler as *mut u64,
            )
        };

        match code {
            0 => Ok(handler),
            _ => Err(Box::new(WasmEdgeError::Instance(
                InstanceError::NotFoundMappedFdHandler,
            ))),
        }
    }
}

// #[cfg(test)]
#[cfg(iginore)]
mod tests {
    use super::*;
    use crate::{
        instance::function::FuncTypeOwn, CallingFrame, Config, Executor, GlobalType, ImportModule,
        MemType, Store, TableType, WasmValue, HOST_FUNCS, HOST_FUNC_FOOTPRINTS,
    };
    #[cfg(not(feature = "async"))]
    use std::sync::{Arc, Mutex};
    use std::thread;
    use wasmedge_macro::sys_host_function;
    use wasmedge_types::{error::HostFuncError, Mutability, NeverType, RefType, ValType};

    #[test]
    // #[cfg(not(feature = "async"))]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_add_instance() {
        assert_eq!(HOST_FUNCS.read().len(), 0);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 0);

        let host_name = "extern";

        // create an import module
        let result = ImportModule::<NeverType>::create(host_name, None);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // create a host function
        let result = FuncTypeOwn::create([ValType::ExternRef, ValType::I32], [ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        let host_func = result.unwrap();
        // add the host function
        import.add_func("func-add", host_func);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // create a table
        let result = TableType::create(RefType::FuncRef, 10, Some(20));
        assert!(result.is_ok());
        let table_ty = result.unwrap();
        let result = Table::create(&table_ty);
        assert!(result.is_ok());
        let host_table = result.unwrap();
        // add the table
        import.add_table("table", host_table);

        // create a memory
        let result = MemType::create(1, Some(2), false);
        assert!(result.is_ok());
        let mem_ty = result.unwrap();
        let result = Memory::create(&mem_ty);
        assert!(result.is_ok());
        let host_memory = result.unwrap();
        // add the memory
        import.add_memory("memory", host_memory);

        // create a global
        let result = GlobalType::create(ValType::I32, Mutability::Const);
        assert!(result.is_ok());
        let global_ty = result.unwrap();
        let result = Global::create(&global_ty, WasmValue::from_i32(666));
        assert!(result.is_ok());
        let host_global = result.unwrap();
        // add the global
        import.add_global("global_i32", host_global);
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_import_module_send() {
        let host_name = "extern";

        // create an ImportModule instance
        let result = ImportModule::<NeverType>::create(host_name, None);
        assert!(result.is_ok());
        let import = result.unwrap();

        let handle = thread::spawn(move || {
            assert!(!import.inner.0.is_null());
            println!("{:?}", import.inner);
        });

        handle.join().unwrap();
    }

    #[test]
    #[cfg(not(feature = "async"))]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_import_module_sync() {
        let host_name = "extern";

        // create an ImportModule instance
        let result = ImportModule::<NeverType>::create(host_name, None);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // add host function
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        import.add_func("add", host_func);

        // add table
        let result = TableType::create(RefType::FuncRef, 0, Some(u32::MAX));
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Table::create(&ty);
        assert!(result.is_ok());
        let table = result.unwrap();
        import.add_table("table", table);

        // add memory
        let memory = {
            let result = MemType::create(10, Some(20), false);
            assert!(result.is_ok());
            let mem_ty = result.unwrap();
            let result = Memory::create(&mem_ty);
            assert!(result.is_ok());
            result.unwrap()
        };
        import.add_memory("memory", memory);

        // add globals
        let result = GlobalType::create(ValType::F32, Mutability::Const);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Global::create(&ty, WasmValue::from_f32(3.5));
        assert!(result.is_ok());
        let global = result.unwrap();
        import.add_global("global", global);

        let import = Arc::new(Mutex::new(import));
        let import_cloned = Arc::clone(&import);
        let handle = thread::spawn(move || {
            let result = import_cloned.lock();
            assert!(result.is_ok());
            let import = result.unwrap();

            // create a store
            let result = Store::create();
            assert!(result.is_ok());
            let mut store = result.unwrap();
            assert!(!store.inner.0.is_null());
            assert!(!store.registered);

            // create an executor
            let result = Config::create();
            assert!(result.is_ok());
            let config = result.unwrap();
            let result = Executor::create(Some(&config), None);
            assert!(result.is_ok());
            let mut executor = result.unwrap();

            // register import object into store
            let result = executor.register_import_module(&mut store, &import);
            assert!(result.is_ok());

            // get the exported module by name
            let result = store.module("extern");
            assert!(result.is_ok());
            let instance = result.unwrap();

            // get the exported function by name
            let result = instance.get_func("add");
            assert!(result.is_ok());

            // get the exported global by name
            let result = instance.get_global("global");
            assert!(result.is_ok());
            let global = result.unwrap();
            assert!(!global.inner.lock().0.is_null() && global.registered);
            let val = global.get_value();
            assert_eq!(val.to_f32(), 3.5);

            // get the exported memory by name
            let result = instance.get_memory("memory");
            assert!(result.is_ok());
            let memory = result.unwrap();
            let result = memory.ty();
            assert!(result.is_ok());
            let ty = result.unwrap();
            assert_eq!(ty.min(), 10);
            assert_eq!(ty.max(), Some(20));

            // get the exported table by name
            let result = instance.get_table("table");
            assert!(result.is_ok());
        });

        handle.join().unwrap();
    }

    #[cfg(all(not(feature = "async"), target_family = "unix"))]
    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_wasi() {
        // create a wasi module instance
        {
            let result = WasiModule::create(None, None, None);
            assert!(result.is_ok());

            let result = WasiModule::create(
                Some(vec!["arg1", "arg2"]),
                Some(vec!["ENV1=VAL1", "ENV1=VAL2", "ENV3=VAL3"]),
                Some(vec![
                    "apiTestData",
                    "Makefile",
                    "CMakeFiles",
                    "ssvmAPICoreTests",
                    ".:.",
                ]),
            );
            assert!(result.is_ok());

            let result = WasiModule::create(
                None,
                Some(vec!["ENV1=VAL1", "ENV1=VAL2", "ENV3=VAL3"]),
                Some(vec![
                    "apiTestData",
                    "Makefile",
                    "CMakeFiles",
                    "ssvmAPICoreTests",
                    ".:.",
                ]),
            );
            assert!(result.is_ok());
            let wasi_import = result.unwrap();
            assert_eq!(wasi_import.exit_code(), 0);
        }
    }

    #[test]
    #[cfg(not(feature = "async"))]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_find_xxx() -> Result<(), Box<dyn std::error::Error>> {
        let module_name = "extern_module";

        // create ImportModule instance
        let result = ImportModule::<NeverType>::create(module_name, None);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // add host function
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        import.add_func("add", host_func);

        // add table
        let result = TableType::create(RefType::FuncRef, 0, Some(u32::MAX));
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Table::create(&ty);
        assert!(result.is_ok());
        let table = result.unwrap();
        import.add_table("table", table);

        // add memory
        let result = MemType::create(0, Some(u32::MAX), false);
        assert!(result.is_ok());
        let mem_ty = result.unwrap();
        let result = Memory::create(&mem_ty);
        assert!(result.is_ok());
        let memory = result.unwrap();
        import.add_memory("mem", memory);

        // add global
        let result = GlobalType::create(ValType::F32, Mutability::Const);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Global::create(&ty, WasmValue::from_f32(3.5));
        assert!(result.is_ok());
        let global = result.unwrap();
        import.add_global("global", global);

        // create an executor
        let mut executor = Executor::create(None, None)?;

        // create a store
        let mut store = Store::create()?;

        executor.register_import_module(&mut store, &import)?;

        // get the module named "extern"
        let result = store.module("extern_module");
        assert!(result.is_ok());
        let instance = result.unwrap();

        // check the name of the module
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), "extern_module");

        // get the exported function named "fib"
        let result = instance.get_func("add");
        assert!(result.is_ok());
        let func = result.unwrap();

        // check the type of the function
        let result = func.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();

        // check the parameter types
        let param_types = ty.params_type_iter().collect::<Vec<ValType>>();
        assert_eq!(param_types, [ValType::I32, ValType::I32]);

        // check the return types
        let return_types = ty.returns_type_iter().collect::<Vec<ValType>>();
        assert_eq!(return_types, [ValType::I32]);

        // get the exported table named "table"
        let result = instance.get_table("table");
        assert!(result.is_ok());
        let table = result.unwrap();

        // check the type of the table
        let result = table.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert_eq!(ty.elem_ty(), RefType::FuncRef);
        assert_eq!(ty.min(), 0);
        assert_eq!(ty.max(), Some(u32::MAX));

        // get the exported memory named "mem"
        let result = instance.get_memory("mem");
        assert!(result.is_ok());
        let memory = result.unwrap();

        // check the type of the memory
        let result = memory.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert_eq!(ty.min(), 0);
        assert_eq!(ty.max(), Some(u32::MAX));

        // get the exported global named "global"
        let result = instance.get_global("global");
        assert!(result.is_ok());
        let global = result.unwrap();

        // check the type of the global
        let result = global.ty();
        assert!(result.is_ok());
        let global = result.unwrap();
        assert_eq!(global.value_type(), ValType::F32);
        assert_eq!(global.mutability(), Mutability::Const);

        Ok(())
    }

    #[test]
    #[cfg(not(feature = "async"))]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_find_names() -> Result<(), Box<dyn std::error::Error>> {
        let module_name = "extern_module";

        // create ImportModule instance
        let result = ImportModule::<NeverType>::create(module_name, None);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // add host function
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        import.add_func("add", host_func);

        // add table
        let result = TableType::create(RefType::FuncRef, 0, Some(u32::MAX));
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Table::create(&ty);
        assert!(result.is_ok());
        let table = result.unwrap();
        import.add_table("table", table);

        // add memory
        let result = MemType::create(0, Some(u32::MAX), false);
        assert!(result.is_ok());
        let mem_ty = result.unwrap();
        let result = Memory::create(&mem_ty);
        assert!(result.is_ok());
        let memory = result.unwrap();
        import.add_memory("mem", memory);

        // add global
        let result = GlobalType::create(ValType::F32, Mutability::Const);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Global::create(&ty, WasmValue::from_f32(3.5));
        assert!(result.is_ok());
        let global = result.unwrap();
        import.add_global("global", global);

        // create an executor
        let mut executor = Executor::create(None, None)?;

        // create a store
        let mut store = Store::create()?;

        executor.register_import_module(&mut store, &import)?;

        // get the module named "extern"
        let result = store.module("extern_module");
        assert!(result.is_ok());
        let instance = result.unwrap();

        // check the name of the module
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), "extern_module");

        assert_eq!(instance.func_len(), 1);
        let result = instance.func_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["add"]);

        assert_eq!(instance.table_len(), 1);
        let result = instance.table_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["table"]);

        assert_eq!(instance.mem_len(), 1);
        let result = instance.mem_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["mem"]);

        assert_eq!(instance.global_len(), 1);
        let result = instance.global_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["global"]);

        Ok(())
    }

    #[test]
    #[cfg(not(feature = "async"))]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_get() {
        let module_name = "extern_module";

        let result = Store::create();
        assert!(result.is_ok());
        let mut store = result.unwrap();
        assert!(!store.inner.0.is_null());
        assert!(!store.registered);

        // check the length of registered module list in store before instantiation
        assert_eq!(store.module_len(), 0);
        assert!(store.module_names().is_none());

        // create ImportObject instance
        let result = ImportModule::<NeverType>::create(module_name, None);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // add host function
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        import.add_func("add", host_func);

        // add table
        let result = TableType::create(RefType::FuncRef, 0, Some(u32::MAX));
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Table::create(&ty);
        assert!(result.is_ok());
        let table = result.unwrap();
        import.add_table("table", table);

        // add memory
        let memory = {
            let result = MemType::create(10, Some(20), false);
            assert!(result.is_ok());
            let mem_ty = result.unwrap();
            let result = Memory::create(&mem_ty);
            assert!(result.is_ok());
            result.unwrap()
        };
        import.add_memory("mem", memory);

        // add globals
        let result = GlobalType::create(ValType::F32, Mutability::Const);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Global::create(&ty, WasmValue::from_f32(3.5));
        assert!(result.is_ok());
        let global = result.unwrap();
        import.add_global("global", global);

        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();
        let result = Executor::create(Some(&config), None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        let result = executor.register_import_module(&mut store, &import);
        assert!(result.is_ok());

        let result = store.module(module_name);
        assert!(result.is_ok());
        let mut instance = result.unwrap();

        // get the exported memory
        let result = instance.get_memory("mem");
        assert!(result.is_ok());
        let memory = result.unwrap();
        let result = memory.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert_eq!(ty.min(), 10);
        assert_eq!(ty.max(), Some(20));

        // get host data
        assert!(instance.host_data::<NeverType>().is_none());
    }

    #[sys_host_function]
    fn real_add(
        _frame: CallingFrame,
        inputs: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, HostFuncError> {
        if inputs.len() != 2 {
            return Err(HostFuncError::User(1));
        }

        let a = if inputs[0].ty() == ValType::I32 {
            inputs[0].to_i32()
        } else {
            return Err(HostFuncError::User(2));
        };

        let b = if inputs[1].ty() == ValType::I32 {
            inputs[1].to_i32()
        } else {
            return Err(HostFuncError::User(3));
        };

        let c = a + b;

        Ok(vec![WasmValue::from_i32(c)])
    }

    #[cfg(not(feature = "async"))]
    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_instance_clone() {
        // clone of ImportModule
        {
            let host_name = "extern";

            // create an import module
            let result = ImportModule::<NeverType>::create(host_name, None);
            assert!(result.is_ok());
            let mut import = result.unwrap();

            // create a host function
            let result = FuncType::create([ValType::ExternRef, ValType::I32], [ValType::I32]);
            assert!(result.is_ok());
            let func_ty = result.unwrap();
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
            assert!(result.is_ok());
            let host_func = result.unwrap();
            // add the host function
            import.add_func("func-add", host_func);

            // create a table
            let result = TableType::create(RefType::FuncRef, 10, Some(20));
            assert!(result.is_ok());
            let table_ty = result.unwrap();
            let result = Table::create(&table_ty);
            assert!(result.is_ok());
            let host_table = result.unwrap();
            // add the table
            import.add_table("table", host_table);

            // create a memory
            let result = MemType::create(1, Some(2), false);
            assert!(result.is_ok());
            let mem_ty = result.unwrap();
            let result = Memory::create(&mem_ty);
            assert!(result.is_ok());
            let host_memory = result.unwrap();
            // add the memory
            import.add_memory("memory", host_memory);

            // create a global
            let result = GlobalType::create(ValType::I32, Mutability::Const);
            assert!(result.is_ok());
            let global_ty = result.unwrap();
            let result = Global::create(&global_ty, WasmValue::from_i32(666));
            assert!(result.is_ok());
            let host_global = result.unwrap();
            // add the global
            import.add_global("global_i32", host_global);
            assert_eq!(Arc::strong_count(&import.inner), 1);

            // clone the import module
            let import_clone = import.clone();
            assert_eq!(Arc::strong_count(&import.inner), 2);

            drop(import);
            assert_eq!(Arc::strong_count(&import_clone.inner), 1);
            drop(import_clone);
        }

        // clone of WasiModule
        {
            let result = WasiModule::create(None, None, None);
            assert!(result.is_ok());

            let result = WasiModule::create(
                Some(vec!["arg1", "arg2"]),
                Some(vec!["ENV1=VAL1", "ENV1=VAL2", "ENV3=VAL3"]),
                Some(vec![
                    "apiTestData",
                    "Makefile",
                    "CMakeFiles",
                    "ssvmAPICoreTests",
                    ".:.",
                ]),
            );
            assert!(result.is_ok());

            let result = WasiModule::create(
                None,
                Some(vec!["ENV1=VAL1", "ENV1=VAL2", "ENV3=VAL3"]),
                Some(vec![
                    "apiTestData",
                    "Makefile",
                    "CMakeFiles",
                    "ssvmAPICoreTests",
                    ".:.",
                ]),
            );
            assert!(result.is_ok());
            let wasi_import = result.unwrap();
            assert_eq!(wasi_import.exit_code(), 0);
            assert_eq!(std::sync::Arc::strong_count(&wasi_import.inner), 1);

            // clone
            let wasi_import_clone = wasi_import.clone();
            assert_eq!(std::sync::Arc::strong_count(&wasi_import.inner), 2);

            drop(wasi_import);
            assert_eq!(std::sync::Arc::strong_count(&wasi_import_clone.inner), 1);
            drop(wasi_import_clone);
        }
    }

    #[test]
    fn test_instance_create_import_with_data() {
        let module_name = "extern_module";

        // define host data
        #[derive(Clone, Debug)]
        struct Circle {
            radius: i32,
        }

        let circle = Circle { radius: 10 };

        // create an import module
        let result = ImportModule::create(module_name, Some(Box::new(circle)));

        assert!(result.is_ok());
        let import = result.unwrap();

        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();
        let result = Executor::create(Some(&config), None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        let result = Store::create();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        let result = executor.register_import_module(&mut store, &import);
        assert!(result.is_ok());

        let result = store.module(module_name);
        assert!(result.is_ok());
        let mut instance = result.unwrap();

        let result = instance.host_data::<Circle>();
        assert!(result.is_some());
        let host_data = result.unwrap();
        assert_eq!(host_data.radius, 10);
    }
}
