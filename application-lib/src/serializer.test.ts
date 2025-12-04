import { expect, test } from "vitest";
import { Deserialize, Serialize } from "./serializer";
import { DataType, Column } from "./typeReflector";
class Vector3 {
  @Column(DataType.Uint64)
  x: number;
  @Column(DataType.Uint64)
  y: number;
  @Column(DataType.Uint64)
  z: number;

  constructor(x?: number, y?: number, z?: number) {
    if (x) this.x = x;
    else this.x = 0;

    if (y) this.y = y;
    else this.y = 0;

    if (z) this.z = z;
    else this.z = 0;
  }
}

test("Serializiation deserializes to same value", () => {
  let vector = new Vector3(555, 888, 222);

  let serialized = Serialize<Vector3>(vector);
  let deserialized = Deserialize(serialized, Vector3);
  expect(deserialized.x).toBe(vector.x);
  expect(deserialized.y).toBe(vector.y);
  expect(deserialized.z).toBe(vector.z);
});
