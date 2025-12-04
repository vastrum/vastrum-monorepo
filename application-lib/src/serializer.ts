import { TypeMetaClass, TypeReflector, DataType } from "./typeReflector";

export function preSerialize<T>(obj: T): TypeMetaClass {
  let p = Object.getPrototypeOf(obj);
  if (p == null || p.constructor.name == "Object") {
    throw new Error("param type is required.");
  }
  let mc = TypeReflector.getMetaClass(p);
  if (mc == null) {
    throw new Error("reflect class: " + p.name + " invalid");
  }
  //mc.sortProperty();
  return mc;
}

export function Serialize<T>(obj: T): string[] {
  let mc = preSerialize(obj);
  return serialize(mc, obj);
}

export function preDeserialize<T>(type: new () => T): [any, TypeMetaClass] {
  let obj = Object.create(type.prototype);
  let mc = TypeReflector.getMetaClass(type.prototype);
  if (mc == null) {
    throw new Error(`reflect class ${type.prototype} invalid.`);
  }
  //mc.sortProperty();
  return [obj, mc];
}

export function Deserialize<T>(payload: string[], type: { new (): T }): T {
  let [obj, mc] = preDeserialize(type);

  return deserialize(payload, obj, mc);
}

function serialize<T>(mc: TypeMetaClass, obj: T) {
  //mc.sortProperty();
  let properties = mc.properties;
  let payload: string[] = [];
  for (let i = 0, len = properties.length; i < len; i++) {
    let p = properties[i];
    payload.push(encode(obj[p.key], p.datatype));
  }
  return payload;
}

function deserialize<T>(payload: string[], obj: T, mc: TypeMetaClass): T {
  if (mc == null) {
    throw new Error("typeMetaClass is null");
  }
  //mc.sortProperty();

  let properties = mc.properties;
  for (let i = 0, len = properties.length; i < len; i++) {
    let p = properties[i];
    obj[p.key] = decode(payload[i], p.datatype);
  }
  return obj;
}

export function encode(val: any, datatype: DataType): string {
  if (datatype == DataType.Bool) {
    return val == true ? "true" : "false";
  } else if (datatype == DataType.String) {
    return val;
  } else if (datatype == DataType.Uint64) {
    return val.toString();
  }
  throw new Error("asd");
}

export function decode(val: string, datatype: DataType): any {
  if (datatype == DataType.Bool) {
    return val == "true" ? true : false;
  } else if (datatype == DataType.String) {
    return val;
  } else if (datatype == DataType.Uint64) {
    return Number(val);
  }
}
