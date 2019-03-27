use cpp_utils::{Ptr, StaticUpcast};
use moqt_core::{QObject, RawSlotOfInt, SlotOfInt};
use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_int, c_void};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn qobject() {
    unsafe {
        let obj1 = QObject::new_0a();
        let obj2 = QObject::new_0a();
        obj1.destroyed().connect(obj2.delete_later());

        let args = QObject::next_connect_args();
        assert_eq!(args.sender().as_ptr(), obj1.as_ptr().as_ptr());
        assert_eq!(args.receiver().as_ptr(), obj2.as_ptr().as_ptr());

        let signal = CStr::from_ptr(args.signal().as_ptr()).to_str().unwrap();
        assert_eq!(signal, "2destroyed(QObject*)");

        let method = CStr::from_ptr(args.method().as_ptr()).to_str().unwrap();
        assert_eq!(method, "1deleteLater()");
    }
}

#[test]
fn raw_slot() {
    unsafe {
        static FLAG: AtomicBool = AtomicBool::new(false);
        extern "C" fn hook(data: *mut c_void, value: c_int) {
            assert_eq!(data, 5 as *mut c_void);
            assert_eq!(value, 7);
            let old = FLAG.swap(true, Ordering::SeqCst);
            assert!(!old);
        }

        let mut obj = RawSlotOfInt::new();
        obj.set(Some(hook), Ptr::new(5 as *mut c_void));
        assert!(!FLAG.load(Ordering::SeqCst));
        obj.custom_slot(7);
        assert!(FLAG.load(Ordering::SeqCst));
    }
}

#[test]
fn raw_slot_connect() {
    unsafe {
        let obj1 = QObject::new_0a();
        let mut slot = RawSlotOfInt::new();
        obj1.object_name_changed().connect(&slot);

        let args = QObject::next_connect_args();
        assert_eq!(args.sender().as_ptr(), obj1.as_ptr().as_ptr());
        let slot_as_qobject: Ptr<QObject> = slot.static_upcast_mut();
        assert_eq!(args.receiver().as_ptr(), slot_as_qobject.as_ptr());

        let signal = CStr::from_ptr(args.signal().as_ptr()).to_str().unwrap();
        assert_eq!(signal, "2objectNameChanged(int)");

        let method = CStr::from_ptr(args.method().as_ptr()).to_str().unwrap();
        assert_eq!(method, "1custom_slot(int)");
    }
}

#[test]
fn closure_slot_connect() {
    unsafe {
        let obj1 = QObject::new_0a();
        let counter = Rc::new(RefCell::new(0));
        let counter_handle = Rc::clone(&counter);
        let mut slot = SlotOfInt::new(move |arg| {
            *counter_handle.borrow_mut() += arg;
        });
        obj1.object_name_changed().connect(&slot);

        let args = QObject::next_connect_args();
        assert_eq!(args.sender().as_ptr(), obj1.as_ptr().as_ptr());

        let signal = CStr::from_ptr(args.signal().as_ptr()).to_str().unwrap();
        assert_eq!(signal, "2objectNameChanged(int)");

        let slot_as_qobject: Ptr<QObject> = slot.as_raw().static_upcast_mut();
        assert_eq!(args.receiver().as_ptr(), slot_as_qobject.as_ptr());

        let method = CStr::from_ptr(args.method().as_ptr()).to_str().unwrap();
        assert_eq!(method, "1custom_slot(int)");

        assert_eq!(*counter.borrow(), 0);
        slot.as_raw().custom_slot(2);
        assert_eq!(*counter.borrow(), 2);
        slot.as_raw().custom_slot(4);
        assert_eq!(*counter.borrow(), 6);

        slot.clear();
        slot.as_raw().custom_slot(8);
        assert_eq!(*counter.borrow(), 6);
    }
}
