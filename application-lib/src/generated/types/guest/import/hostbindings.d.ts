/// <reference path="./interfaces/contractbindings.d.ts" />
/// <reference path="./interfaces/runtimebindings.d.ts" />
declare module 'volans:component/hostbindings' {
  export type * as Runtimebindings from 'runtimebindings'; // import runtimebindings
  export * as contractbindings from 'contractbindings'; // export contractbindings
}
