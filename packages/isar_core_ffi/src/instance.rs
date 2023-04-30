use crate::{
    require_from_c_str, CIsarCursor, CIsarInsert, CIsarInstance, CIsarQuery, CIsarQueryBuilder,
    CIsarTxn,
};
use isar_core::core::instance::{CompactCondition, IsarInstance};
use isar_core::core::schema::IsarSchema;
use isar_core::native::native_instance::NativeInstance;
use std::os::raw::c_char;
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

#[no_mangle]
pub unsafe extern "C" fn isar_version() -> *const c_char {
    ISAR_VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn isar_get(instance_id: u32) -> *const CIsarInstance {
    if let Some(instance) = NativeInstance::get(instance_id) {
        Box::into_raw(Box::new(CIsarInstance::Native(instance)))
    } else {
        ptr::null()
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_create(
    isar: *mut *const CIsarInstance,
    instance_id: u32,
    name: *const c_char,
    path: *const c_char,
    schema_json: *const c_char,
    max_size_mib: u32,
    relaxed_durability: bool,
    compact_min_file_size: u32,
    compact_min_bytes: u32,
    compact_min_ratio: f32,
) -> u8 {
    isar_try! {
        let name = require_from_c_str(name)?;
        let path = require_from_c_str(path)?;
        let schema_json = require_from_c_str(schema_json)?;
        let schema = IsarSchema::from_json(schema_json.as_bytes())?;

        let compact_condition = if compact_min_ratio.is_nan() {
            None
        } else {
            Some(CompactCondition {
                min_file_size: compact_min_file_size ,
                min_bytes: compact_min_bytes,
                min_ratio: compact_min_ratio,
            })
        };

        let native_instance = NativeInstance::open(
            instance_id ,
            name,
            path,
            schema,
            max_size_mib,
            relaxed_durability,
            compact_condition,
        )?;
        let new_isar = CIsarInstance::Native(native_instance);
        *isar = Box::into_raw(Box::new(new_isar));
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_txn_begin(
    isar: &'static CIsarInstance,
    txn: *mut *const CIsarTxn,
    write: bool,
) -> u8 {
    isar_try! {
        let new_txn = match isar {
            CIsarInstance::Native(isar) =>{
                let txn = isar.begin_txn(write)?;
                CIsarTxn::Native(txn)
            },
        };
        *txn = Box::into_raw(Box::new(new_txn));
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_txn_commit(isar: &'static CIsarInstance, txn: *mut CIsarTxn) -> u8 {
    isar_try! {
        let txn = *Box::from_raw(txn);
        match (isar, txn) {
           (CIsarInstance::Native(isar), CIsarTxn::Native(txn)) => {
                isar.commit_txn(txn)?;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_txn_abort(isar: &'static CIsarInstance, txn: *mut CIsarTxn) {
    let txn = *Box::from_raw(txn);
    match (isar, txn) {
        (CIsarInstance::Native(isar), CIsarTxn::Native(txn)) => {
            isar.abort_txn(txn);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_insert(
    isar: &'static CIsarInstance,
    txn: *mut CIsarTxn,
    insert: *mut *const CIsarInsert,
    collection_index: u16,
    count: u32,
) -> u8 {
    isar_try! {
        let txn = *Box::from_raw(txn);
        let new_insert = match (isar, txn) {
            (CIsarInstance::Native(isar), CIsarTxn::Native(txn)) => {
                let insert = isar.insert(txn, collection_index, count)?;
                CIsarInsert::Native(insert)
            }
        };
        *insert = Box::into_raw(Box::new(new_insert));
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_build_query(
    isar: &'static CIsarInstance,
    query_builder: *mut *const CIsarQueryBuilder,
    collection_index: u16,
) -> u8 {
    isar_try! {
        let new_query_builder = match isar {
            CIsarInstance::Native(isar) => {
                let query_builder = isar.build_query(collection_index)?;
                CIsarQueryBuilder::Native(query_builder)
            }
        };
        *query_builder = Box::into_raw(Box::new(new_query_builder));
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_query(
    isar: &'static CIsarInstance,
    txn: &'static CIsarTxn,
    query: &'static CIsarQuery,
    cursor: *mut *const CIsarCursor,
) -> u8 {
    isar_try! {
        let new_cursor = match (isar,txn,query) {
            (CIsarInstance::Native(isar), CIsarTxn::Native(txn),CIsarQuery::Native(query)) => {
                let cursor = isar.query(txn,query)?;
                CIsarCursor::Native(cursor)
            }
        };
        *cursor = Box::into_raw(Box::new(new_cursor));
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_query_count(
    isar: &'static CIsarInstance,
    txn: &'static CIsarTxn,
    query: &'static CIsarQuery,
    count: *mut u32,
) -> u8 {
    isar_try! {
        let new_count = match (isar,txn,query) {
            (CIsarInstance::Native(isar), CIsarTxn::Native(txn),CIsarQuery::Native(query)) => {
                isar.count(txn,query)?
            }
        };
        *count = new_count;
    }
}

#[no_mangle]
pub unsafe extern "C" fn isar_query_delete(
    isar: &'static CIsarInstance,
    txn: &'static CIsarTxn,
    query: &'static CIsarQuery,
    count: *mut u32,
) -> u8 {
    isar_try! {
        let new_count = match (isar,txn,query) {
            (CIsarInstance::Native(isar), CIsarTxn::Native(txn),CIsarQuery::Native(query)) => {
                isar.delete(txn,query)?
            }
        };
        *count = new_count;
    }
}
