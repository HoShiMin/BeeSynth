use windows::Win32::{
    System::{ Registry::{ HKEY, RegCloseKey }, Services::CloseServiceHandle },
    Foundation::{ HANDLE, CloseHandle, INVALID_HANDLE_VALUE },
    Security::SC_HANDLE
};

pub trait ObjectTrait {
    type Object: Copy;
    const INVALID: Self::Object;
    fn is_valid(obj: Self::Object) -> bool;
    fn close(obj: &mut Self::Object);
}

pub struct HandleTrait;

impl ObjectTrait for HandleTrait {
    type Object = HANDLE;

    const INVALID: HANDLE = HANDLE(0);

    fn is_valid(obj: HANDLE) -> bool {
        obj != Self::INVALID
    }

    fn close(obj: &mut HANDLE) {
        unsafe { CloseHandle(*obj) };
        *obj = Self::INVALID;
    }
}

pub struct FileHandleTrait;

impl ObjectTrait for FileHandleTrait {
    type Object = HANDLE;

    const INVALID: HANDLE = INVALID_HANDLE_VALUE;

    fn is_valid(obj: HANDLE) -> bool {
        obj != Self::INVALID
    }

    fn close(obj: &mut HANDLE) {
        unsafe { CloseHandle(*obj) };
        *obj = Self::INVALID;
    }
}

pub struct RegKeyTrait;

impl ObjectTrait for RegKeyTrait {
    type Object = HKEY;

    const INVALID: HKEY = HKEY(0);

    fn is_valid(obj: HKEY) -> bool {
        obj != Self::INVALID
    }

    fn close(obj: &mut HKEY) {
        unsafe { RegCloseKey(*obj) };
        *obj = Self::INVALID;
    }
}

pub struct ServiceHandleTrait;

impl ObjectTrait for ServiceHandleTrait {
    type Object = SC_HANDLE;

    const INVALID: SC_HANDLE = SC_HANDLE(0);

    fn is_valid(obj: Self::Object) -> bool {
        obj != Self::INVALID
    }

    fn close(obj: &mut Self::Object) {
        unsafe { CloseServiceHandle(*obj) };
        *obj = Self::INVALID;
    }
}



pub struct Auto<ObjTrait: ObjectTrait> {
    obj: ObjTrait::Object
}

impl<ObjTrait: ObjectTrait> Default for Auto<ObjTrait> {
    fn default() -> Self {
        Self {
            obj: ObjTrait::INVALID
        }
    }
}

impl<ObjTrait: ObjectTrait> Auto<ObjTrait> {
    pub fn new(obj: ObjTrait::Object) -> Self {
        Self { obj }
    }

    pub fn is_valid(&self) -> bool {
        ObjTrait::is_valid(self.obj)
    }

    pub fn close(&mut self) {
        if self.is_valid() {
            ObjTrait::close(&mut self.obj);
        }
    }

    #[must_use]
    pub fn get(&self) -> ObjTrait::Object {
        self.obj
    }

    pub fn set(&mut self, obj: ObjTrait::Object) {
        self.close();
        self.obj = obj;
    }

    #[must_use]
    pub fn detach(mut self) -> ObjTrait::Object {
        std::mem::replace(&mut self.obj, ObjTrait::INVALID)
    }
}

impl<ObjTrait: ObjectTrait> Drop for Auto<ObjTrait> {
    fn drop(&mut self) {
        self.close();
    }
}


pub type Handle = Auto<HandleTrait>;
pub type FileHandle = Auto<FileHandleTrait>;
pub type RegKey = Auto<RegKeyTrait>;
pub type ServiceHandle = Auto<ServiceHandleTrait>;

impl From<HANDLE> for Handle {
    fn from(value: HANDLE) -> Self {
        Self::new(value)
    }
}

impl From<HANDLE> for FileHandle {
    fn from(value: HANDLE) -> Self {
        Self::new(value)
    }
}

impl From<HKEY> for RegKey {
    fn from(value: HKEY) -> Self {
        Self::new(value)
    }
}

impl From<SC_HANDLE> for ServiceHandle {
    fn from(value: SC_HANDLE) -> Self {
        Self::new(value)
    }
}

pub trait HandleWrapper {
    fn wrap_handle(self) -> Handle;
}

pub trait FileHandleWrapper {
    fn wrap_file_handle(self) -> FileHandle;
}

pub trait RegKeyWrapper {
    fn wrap_reg_key(self) -> RegKey;
}

pub trait ServiceHandleWrapper {
    fn wrap_service_handle(self) -> ServiceHandle;
}

impl HandleWrapper for HANDLE {
    fn wrap_handle(self) -> Handle {
        Handle::new(self)
    }
}

impl FileHandleWrapper for HANDLE {
    fn wrap_file_handle(self) -> FileHandle {
        FileHandle::new(self)
    }
}

impl RegKeyWrapper for HKEY {
    fn wrap_reg_key(self) -> RegKey {
        RegKey::new(self)
    }
}

impl ServiceHandleWrapper for SC_HANDLE {
    fn wrap_service_handle(self) -> ServiceHandle {
        ServiceHandle::new(self)
    }
}