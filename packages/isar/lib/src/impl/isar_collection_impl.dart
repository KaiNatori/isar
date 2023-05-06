part of isar;

class _IsarCollectionImpl<ID, OBJ> implements IsarCollection<ID, OBJ> {
  const _IsarCollectionImpl(this.isar, this.collectionIndex, this.converter);

  final _IsarImpl isar;
  final int collectionIndex;
  final ObjectConverter<ID, OBJ> converter;

  Pointer<CIsarQuery> _idQuery(ID id) {
    final queryPtr = IsarCore.ptrPtr.cast<Pointer<CIsarQuery>>();
    IsarCore.isar_query_new_id(
      isar.ptr,
      collectionIndex,
      _idToInt(id),
      queryPtr,
    ).checkNoError();
    return queryPtr.value;
  }

  Pointer<CIsarQuery> _idsQuery(List<ID> ids) {
    final queryPtr = IsarCore.ptrPtr.cast<Pointer<CIsarQuery>>();
    final idsPtr = malloc<Int64>(ids.length);
    for (var i = 0; i < ids.length; i++) {
      idsPtr[i] = _idToInt(ids[i]);
    }
    final result = IsarCore.isar_query_new_ids(
      isar.ptr,
      collectionIndex,
      idsPtr,
      ids.length,
      queryPtr,
    );
    malloc.free(idsPtr);

    result.checkNoError();
    return queryPtr.value;
  }

  @override
  OBJ? get(ID id) {
    return isar.getTxn((txnPtr) {
      final queryPtr = _idQuery(id);
      final cursor = _RawCursor(
        isarPtr: isar.ptr,
        txnPtr: txnPtr,
        queryPtr: queryPtr,
        deserialize: converter.deserialize,
        offset: 0,
        limit: 1,
      );
      return cursor.findFirst();
    });
  }

  @override
  List<OBJ> getAll(List<ID> ids) {
    return isar.getTxn((txnPtr) {
      final queryPtr = _idsQuery(ids);
      final cursor = _RawCursor(
        isarPtr: isar.ptr,
        txnPtr: txnPtr,
        queryPtr: queryPtr,
        deserialize: converter.deserialize,
        offset: 0,
        limit: ids.length,
      );
      return cursor.findAll();
    });
  }

  @override
  void put(OBJ object) {
    putAll([object]);
  }

  @override
  void putAll(List<OBJ> objects) {
    return isar.getWriteTxn(consume: true, (txnPtr) {
      final insertPtrPtr = IsarCore.ptrPtr.cast<Pointer<CIsarInsert>>();
      IsarCore.isar_insert(
        isar.ptr,
        txnPtr,
        collectionIndex,
        objects.length,
        insertPtrPtr,
      ).checkNoError();

      for (final object in objects) {
        final writerPtr = insertPtrPtr.value.cast<CIsarWriter>();
        converter.serialize(object, writerPtr);
        IsarCore.isar_insert_save(insertPtrPtr, converter.getId(object))
            .checkNoError();
      }

      final insertPtr = insertPtrPtr.value;
      final txnPtrPtr = IsarCore.ptrPtr.cast<Pointer<CIsarTxn>>();
      IsarCore.isar_insert_finish(insertPtr, txnPtrPtr).checkNoError();

      return (null, txnPtrPtr.value);
    });
  }

  @override
  bool delete(ID id) {
    return deleteAll([id]) == 1;
  }

  @override
  int deleteAll(List<ID> id) {
    throw UnimplementedError();
  }

  @override
  QueryBuilder<OBJ, OBJ, QFilter> where() {
    throw UnimplementedError();
  }

  @override
  int count() {
    return isar.getTxn((txnPtr) {
      IsarCore.isar_count(isar.ptr, txnPtr, collectionIndex, IsarCore.countPtr);
      return IsarCore.countPtr.value;
    });
  }

  @override
  void clear() {}

  @override
  Query<R> buildQuery<R>({
    Filter? filter,
    List<SortProperty> sortBy = const [],
    List<DistinctProperty> distinctBy = const [],
    int? property,
  }) {
    final alloc = Arena(malloc);
    final builderPtrPtr = alloc<Pointer<CIsarQueryBuilder>>();
    IsarCore.isar_query_new(isar.ptr, collectionIndex, builderPtrPtr)
        .checkNoError();

    final builderPtr = builderPtrPtr.value;
    if (filter != null) {
      final filterPtr = buildFilter(alloc, filter);
      IsarCore.isar_query_set_filter(builderPtr, filterPtr);
    }

    for (final sort in sortBy) {
      IsarCore.isar_query_add_sort(
        builderPtr,
        sort.property,
        sort.sort == Sort.asc,
      );
    }

    final query = IsarCore.isar_query_build(builderPtr);
    throw UnimplementedError();
  }
}

@pragma('vm:prefer-inline')
int _idToInt<OBJ>(OBJ id) {
  if (id is int) {
    return id;
  } else if (id is String) {
    return Isar.fastHash(id);
  } else {
    throw 'Unsupported id type';
  }
}
