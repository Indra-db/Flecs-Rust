use crate::core::*;

pub(crate) fn get_rw_lock<T: GetTuple, Return, const MULTITHREADED: bool>(
    world: &WorldRef,
    callback: impl FnOnce(<T as GetTuple>::TupleType<'_>) -> Return,
    tuple_data: <T as GetTuple>::Pointers,
    tuple: <T as GetTuple>::TupleType<'_>,
) -> Return {
    let components = tuple_data.component_ptrs();
    let safety_info = tuple_data.safety_info();
    let world = world.real_world();
    let stage_id = if MULTITHREADED {
        world.stage_id()
    } else {
        0 // stage_id is not used in single-threaded mode
    };

    for (index, si) in safety_info.iter().enumerate() {
        use crate::core::SafetyInfo;

        if unsafe { components.get_unchecked(index).is_null() } {
            continue;
        }
        match si {
            SafetyInfo::Read(si) => {
                if !si.cr.is_null() {
                    use crate::core::sparse_id_record_lock_read_begin;

                    sparse_id_record_lock_read_begin::<MULTITHREADED>(&world, si.cr);
                } else {
                    use crate::core::get_table_column_lock_read_begin;

                    get_table_column_lock_read_begin::<MULTITHREADED>(
                        &world,
                        si.table,
                        si.column_index,
                        stage_id,
                    );
                }
            }
            SafetyInfo::Write(si) => {
                if !si.cr.is_null() {
                    sparse_id_record_lock_write_begin::<MULTITHREADED>(&world, si.cr);
                } else {
                    get_table_column_lock_write_begin::<MULTITHREADED>(
                        &world,
                        si.table,
                        si.column_index,
                        stage_id,
                    );
                }
            }
        }
    }

    world.defer_begin();
    let ret = callback(tuple);
    world.defer_end();

    for (index, si) in safety_info.iter().enumerate() {
        if unsafe { components.get_unchecked(index).is_null() } {
            continue;
        }
        match si {
            SafetyInfo::Read(si) => {
                if !si.cr.is_null() {
                    sparse_id_record_lock_read_end::<MULTITHREADED>(si.cr);
                } else {
                    table_column_lock_read_end::<MULTITHREADED>(
                        si.table,
                        si.column_index,
                        stage_id,
                    );
                }
            }
            SafetyInfo::Write(si) => {
                if !si.cr.is_null() {
                    sparse_id_record_lock_write_end::<MULTITHREADED>(si.cr);
                } else {
                    table_column_lock_write_end::<MULTITHREADED>(
                        si.table,
                        si.column_index,
                        stage_id,
                    );
                }
            }
        }
    }
    ret
}
