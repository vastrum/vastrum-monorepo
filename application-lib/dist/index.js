import { blocktime, dbcreatetable, dbinsertentry, dbquery, dbupdateentry, registerstaticroute } from "runtimebindings";

//#region src/typeReflector.ts
let DataType = /* @__PURE__ */ function(DataType$1) {
	DataType$1[DataType$1["Bool"] = 0] = "Bool";
	DataType$1[DataType$1["Uint64"] = 1] = "Uint64";
	DataType$1[DataType$1["String"] = 2] = "String";
	return DataType$1;
}({});
var MetadataStorage = class {
	entities = /* @__PURE__ */ new Map();
	entitiesAmount = 0;
	getEntityMetadata(target) {
		let entityMetadata = this.entities.get(target);
		if (!entityMetadata) {
			let new_entityMetadata = {
				target,
				tableID: this.entitiesAmount,
				columns: [],
				columnToIDMapping: /* @__PURE__ */ new Map(),
				currentColumnID: 0,
				amountOfColumns: 0
			};
			this.entities.set(target, new_entityMetadata);
			this.entitiesAmount += 1;
			return new_entityMetadata;
		}
		return entityMetadata;
	}
	addColumn(target, columnMetadata) {
		let entityMetadata = this.getEntityMetadata(target);
		entityMetadata.columns.push(columnMetadata);
		entityMetadata.columnToIDMapping.set(columnMetadata.propertyName, entityMetadata.currentColumnID);
		entityMetadata.currentColumnID = entityMetadata.currentColumnID + 1;
		entityMetadata.amountOfColumns = entityMetadata.amountOfColumns + 1;
	}
};
const metadataStorage = new MetadataStorage();
function Column(type, array = false, ptype) {
	return function(target, propertyName) {
		metadataStorage.addColumn(target.constructor, {
			propertyName,
			type
		});
		TypeReflector.register(target, propertyName, type, array, ptype);
	};
}
var TypeMetaProperty = class {
	key;
	datatype;
	isArray = false;
	pclass;
	constructor(key, datatype, isary = false, pclass) {
		this.key = key;
		this.datatype = datatype;
		this.isArray = isary;
		this.pclass = pclass;
	}
};
var TypeMetaClass = class {
	get prototype() {
		return this.m_prototype;
	}
	set prototype(v) {
		this.m_prototype = v;
		this.pname = this.protoName;
	}
	m_prototype;
	properties;
	pname;
	m_needSort = true;
	addProperty(k, t, isary = false) {
		let p = this.properties;
		for (let i = 0, len = p.length; i < len; i++) if (p[i].key == k) return;
		p.push(new TypeMetaProperty(k, t, isary));
		this.m_needSort = true;
	}
	get protoName() {
		return this.prototype.constructor.name;
	}
	sortProperty() {}
};
var TypeReflector = class TypeReflector {
	static meta = [];
	static registerInternal(type) {
		var prototype = type.prototype;
		let meta = TypeReflector.meta;
		for (var i = 0, len = meta.length; i < len; i++) if (meta[i].prototype === prototype) return;
		var metaclass = new TypeMetaClass();
		metaclass.prototype = prototype;
		TypeReflector.meta.push(metaclass);
	}
	static register(proto, property, type, array = false, ptype) {
		let metaclass = TypeReflector.getMetaClass(proto);
		if (metaclass == null) {
			metaclass = new TypeMetaClass();
			metaclass.prototype = proto;
			metaclass.properties = [];
			TypeReflector.meta.push(metaclass);
		}
		let mp = new TypeMetaProperty(property, type, array);
		metaclass.properties.push(mp);
	}
	static getMetaClass(prototype) {
		let meta = TypeReflector.meta;
		for (var i = 0, len = meta.length; i < len; i++) {
			let m = meta[i];
			if (m.prototype === prototype) return m;
		}
		return null;
	}
};

//#endregion
//#region src/serializer.ts
function preDeserialize(type) {
	let obj = Object.create(type.prototype);
	let mc = TypeReflector.getMetaClass(type.prototype);
	if (mc == null) throw new Error(`reflect class ${type.prototype} invalid.`);
	return [obj, mc];
}
function Deserialize(payload, type) {
	let [obj, mc] = preDeserialize(type);
	return deserialize(payload, obj, mc);
}
function deserialize(payload, obj, mc) {
	if (mc == null) throw new Error("typeMetaClass is null");
	let properties = mc.properties;
	for (let i = 0, len = properties.length; i < len; i++) {
		let p = properties[i];
		obj[p.key] = decode(payload[i], p.datatype);
	}
	return obj;
}
function encode(val, datatype) {
	if (datatype == DataType.Bool) return val == true ? "true" : "false";
	else if (datatype == DataType.String) return val;
	else if (datatype == DataType.Uint64) return val.toString();
	throw new Error("asd");
}
function decode(val, datatype) {
	if (datatype == DataType.Bool) return val == "true" ? true : false;
	else if (datatype == DataType.String) return val;
	else if (datatype == DataType.Uint64) return Number(val);
}

//#endregion
//#region src/dblib.ts
var TypedQuery = class TypedQuery {
	whereOperations = [];
	sortingOperations = [];
	limitCount = 0;
	offsetCount = 0;
	constructor(repositoryMetaData, entityClass) {
		this.repositoryMetaData = repositoryMetaData;
		this.entityClass = entityClass;
	}
	where(field, operator, value) {
		const newQuery = this.clone();
		let columnID = this.repositoryMetaData.columnToIDMapping.get(field.toString());
		if (columnID != void 0) newQuery.whereOperations.push({
			field_id: columnID,
			value: value.toString()
		});
		return newQuery;
	}
	orderBy(field, direction = "asc") {
		const newQuery = this.clone();
		let columnID = this.repositoryMetaData.columnToIDMapping.get(field.toString());
		if (columnID != void 0) newQuery.sortingOperations.push({
			field_id: columnID,
			descending: direction == "desc"
		});
		return newQuery;
	}
	limit(count) {
		const newQuery = this.clone();
		newQuery.limitCount = count;
		return newQuery;
	}
	offset(count) {
		const newQuery = this.clone();
		newQuery.offsetCount = count;
		return newQuery;
	}
	get() {
		let result = db_query({
			table_id: this.repositoryMetaData.tableID,
			number_of_fields: this.repositoryMetaData.amountOfColumns,
			sorting_operations: this.sortingOperations,
			where_operations: this.whereOperations,
			limit: this.limitCount,
			offset: this.offsetCount
		});
		let parsed_results = [];
		for (let i = 0; i < result.length; i++) {
			let row = result[i];
			parsed_results.push(Deserialize(row, this.entityClass));
		}
		return parsed_results;
	}
	clone() {
		const newQuery = new TypedQuery(this.repositoryMetaData, this.entityClass);
		newQuery.whereOperations = this.whereOperations;
		newQuery.sortingOperations = this.sortingOperations;
		newQuery.limitCount = this.limitCount;
		newQuery.offsetCount = this.offsetCount;
		return newQuery;
	}
};
function createFieldBindings() {
	return new Proxy({}, { get(target, prop) {
		return prop;
	} });
}
var Repository = class {
	metadata;
	columns;
	constructor(entityClass, tableID) {
		this.entityClass = entityClass;
		this.tableID = tableID;
		this.metadata = metadataStorage.getEntityMetadata(this.entityClass);
		this.columns = createFieldBindings();
	}
	query() {
		return new TypedQuery(this.metadata, this.entityClass);
	}
	insert(value) {
		let data = [];
		for (let i = 0; i < this.metadata.columns.length; i++) {
			let column = this.metadata.columns[i];
			data.push(encode(value[column.propertyName], column.type));
		}
		db_insert_entry({
			data,
			table_id: this.metadata.tableID
		});
	}
	update(select_on_primary_key_value, value) {
		let data = [];
		for (let i = 1; i < this.metadata.columns.length; i++) {
			let column = this.metadata.columns[i];
			data.push(encode(value[column.propertyName], column.type));
		}
		db_update_entry({
			select_on_primary_key_value: select_on_primary_key_value.toString(),
			data,
			table_id: this.metadata.tableID
		});
	}
	createTable() {
		let tableFields = [];
		for (let i = 0; i < this.metadata.columns.length; i++) {
			this.metadata.columns[i];
			tableFields.push(SQLFieldTypes.Text);
		}
		db_create_table({
			table_id: this.tableID,
			fields: tableFields
		});
	}
};

//#endregion
//#region src/index.ts
function register_static_route(route, html) {
	registerstaticroute(route, html);
}
function register_route(route, data, template_path) {}
function block_time() {
	return Number(blocktime());
}
let SQLFieldTypes = /* @__PURE__ */ function(SQLFieldTypes$1) {
	SQLFieldTypes$1[SQLFieldTypes$1["Integer"] = 0] = "Integer";
	SQLFieldTypes$1[SQLFieldTypes$1["Text"] = 1] = "Text";
	SQLFieldTypes$1[SQLFieldTypes$1["Blob"] = 2] = "Blob";
	return SQLFieldTypes$1;
}({});
function db_insert_entry(databaseInsertEntry) {
	dbinsertentry(JSON.stringify(databaseInsertEntry));
}
function db_update_entry(databaseUpdateEntry) {
	dbupdateentry(JSON.stringify(databaseUpdateEntry));
}
function db_query(databaseQuery) {
	let jsonresponse = dbquery(JSON.stringify(databaseQuery));
	return JSON.parse(jsonresponse).result;
}
function db_create_table(databaseCreateTable) {
	dbcreatetable(JSON.stringify(databaseCreateTable));
}

//#endregion
export { Column, DataType, Repository, SQLFieldTypes, TypedQuery, block_time, db_create_table, db_insert_entry, db_query, db_update_entry, register_route, register_static_route };