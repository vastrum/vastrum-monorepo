import type { u64 } from ".";

export enum DataType {
  Bool,
  Uint64,
  String,
}

export interface ColumnMetadata {
  propertyName: string;
  type: DataType;
}

export interface RepositoryMetadata {
  target: Function;
  tableID: u64; //actual id used in backend
  columns: ColumnMetadata[];
  columnToIDMapping: Map<String, u64>;
  currentColumnID: u64;
  amountOfColumns: u64;
}

class MetadataStorage {
  private entities: Map<Function, RepositoryMetadata> = new Map();
  private entitiesAmount = 0;

  getEntityMetadata(target: Function): RepositoryMetadata {
    let entityMetadata = this.entities.get(target);
    if (!entityMetadata) {
      let new_entityMetadata = {
        target: target,
        tableID: this.entitiesAmount,
        columns: [],
        columnToIDMapping: new Map(),
        currentColumnID: 0,
        amountOfColumns: 0,
      };
      this.entities.set(target, new_entityMetadata);
      this.entitiesAmount += 1;
      return new_entityMetadata;
    }
    return entityMetadata;
  }

  addColumn(target: Function, columnMetadata: ColumnMetadata): void {
    let entityMetadata = this.getEntityMetadata(target);
    entityMetadata.columns.push(columnMetadata);
    entityMetadata.columnToIDMapping.set(
      columnMetadata.propertyName,
      entityMetadata.currentColumnID
    );
    entityMetadata.currentColumnID = entityMetadata.currentColumnID + 1;
    entityMetadata.amountOfColumns = entityMetadata.amountOfColumns + 1;
  }
}

export const metadataStorage = new MetadataStorage();

export function Column(
  type: DataType,
  array: boolean = false,
  ptype?: any | DataType
) {
  return function (target: any, propertyName: string) {
    metadataStorage.addColumn(target.constructor, {
      propertyName,
      type,
    });
    TypeReflector.register(target, propertyName, type, array, ptype);
  };
}

export class TypeMetaProperty {
  public key: string;
  public datatype: DataType;
  public isArray: boolean = false;
  public pclass?: TypeMetaClass | DataType;

  public constructor(
    key: string,
    datatype: DataType,
    isary: boolean = false,
    pclass?: any
  ) {
    this.key = key;
    this.datatype = datatype;
    this.isArray = isary;
    this.pclass = pclass;
  }
}

function sort(a: TypeMetaProperty, b: TypeMetaProperty) {
  return a.key.localeCompare(b.key);
}

export class TypeMetaClass {
  public get prototype(): any {
    return this.m_prototype;
  }

  public set prototype(v: any) {
    this.m_prototype = v;
    this.pname = this.protoName;
  }
  private m_prototype: any;
  public properties: TypeMetaProperty[];

  public pname: string;

  private m_needSort: boolean = true;

  public addProperty(k: string, t: DataType, isary: boolean = false) {
    let p = this.properties;
    for (let i = 0, len = p.length; i < len; i++) {
      if (p[i].key == k) return;
    }
    p.push(new TypeMetaProperty(k, t, isary));
    this.m_needSort = true;
  }

  public get protoName(): string {
    return this.prototype.constructor.name;
  }

  public sortProperty() {
    /*if (!this.m_needSort) return;
    this.properties.sort((a, b) => sort(a, b));
    this.m_needSort = false;
    return this;
    */
  }
}

export class TypeReflector {
  public static meta: TypeMetaClass[] = [];

  public static registerInternal(type: any) {
    var prototype = type.prototype;
    let meta = TypeReflector.meta;
    for (var i = 0, len = meta.length; i < len; i++) {
      let m = meta[i];
      if (m.prototype === prototype) {
        return;
      }
    }

    var metaclass = new TypeMetaClass();
    metaclass.prototype = prototype;
    TypeReflector.meta.push(metaclass);
  }

  public static register(
    proto: any,
    property: string,
    type: DataType,
    array: boolean = false,
    ptype?: any
  ) {
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

  public static getMetaClass(prototype: any): TypeMetaClass | null {
    let meta = TypeReflector.meta;
    for (var i = 0, len = meta.length; i < len; i++) {
      let m = meta[i];
      if (m.prototype === prototype) {
        return m;
      }
    }
    return null;
  }
}
