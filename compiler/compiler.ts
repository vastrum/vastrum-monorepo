import { keccak256 } from "js-sha3";
import {
  ArrayTypeNode,
  ArrowFunction,
  FunctionDeclaration,
  FunctionExpression,
  MethodDeclaration,
  ParameterDeclaration,
  Project,
  ScriptKind,
  SourceFile,
  ts,
  Type,
} from "ts-morph";
import { writeFileSync, existsSync, mkdirSync, readFileSync } from "fs";
import { dirname, join } from "path";

export type u64 = number;
export type i64 = number;
//export type string = string;
//export type boolean = boolean;

type functionParameter = {
  name: string;
  typeText: string;
  inferredType: string;
  isRestParameter: boolean;
  isOptionalParameter: boolean;
  isParameterProperty: boolean;
  isArray: boolean;
  arrayType: string;
  isTuple: boolean;
};
type publicInterface = {
  function_signature: string;
  raw_function_signature: string;
  functionName: string;
  parameters: functionParameter[];
};

const CALLDATA = "CALLDATA";

parse();

function get_cmd_input() {
  const args = process.argv.slice(2);
  const sourceFilePath = args[0];
  const outFilePath = args[1];
  if (!sourceFilePath || !outFilePath) {
    throw new Error(`No command argument for file path provided`);
  }
  return { sourceFilePath: sourceFilePath, outFilePath: outFilePath };
}
function parse() {
  const input = get_cmd_input();
  const sourceFilePath = input.sourceFilePath;
  const outFilePath = input.outFilePath;
  const project = new Project();
  project.addSourceFilesAtPaths("**/*.ts");

  const sourceFile = project.getSourceFileOrThrow(sourceFilePath);
  parse_public_interface(sourceFile, sourceFilePath, outFilePath);
}

function parse_public_interface(
  sourceFile: SourceFile,
  sourceFilePath: string,
  outFilePath: string
) {
  const hasClasses = sourceFile.getClasses().length > 0;
  const functions = sourceFile.getFunctions();

  let publicInterface = [];
  for (const func of functions) {
    const isExported = func.isExported();

    if (isExported) {
      const functionName = func.getName();
      const parameters = getParameterTypes(func);

      let param_sig: string = "";
      const supportedParameterTypes: Record<string, boolean> = {
        u32: true,
        u64: true,
        i32: true,
        i64: true,
        string: true,
        boolean: true,
      };
      parameters.forEach((param, index) => {
        //https://ts-morph.com/details/parameters
        const isRestParameter = param.isRestParameter;
        const isOptionalParameter = param.isOptionalParameter;
        const isParameterProperty = param.isParameterProperty;
        const isArray = param.isArray;
        const isTuple = param.isTuple;
        if (isTuple) {
          throw new Error(`isTuple`);
        }
        if (isRestParameter) {
          throw new Error(`isRestParameter`);
        }
        if (isOptionalParameter) {
          throw new Error(`isOptionalParameter`);
        }
        if (isParameterProperty) {
          throw new Error("isParameterProperty");
        }
        if (isArray) {
          const isSupportedArrayType = supportedParameterTypes[param.arrayType];
          if (!isSupportedArrayType) {
            const line = func.getEndLineNumber();
            const supported = Object.keys(supportedParameterTypes).join(", ");
            throw new Error(
              `Type ${param.arrayType} is not supported as array type : Supported types (${supported}) line: ${line}`
            );
          }
        } else {
          const isSupportedType = supportedParameterTypes[param.typeText];
          if (!isSupportedType) {
            const line = func.getEndLineNumber();
            const supported = Object.keys(supportedParameterTypes).join(", ");
            throw new Error(
              `Type ${param.typeText} is not supported as interface type : Supported types (${supported}) line: ${line}`
            );
          }
        }

        if (index != 0) {
          param_sig += ",";
        }
        param_sig += param.typeText;
      });
      const raw_function_signature = `${functionName}(${param_sig})`;
      let function_signature: string = keccak256(raw_function_signature);
      function_signature = `0x${function_signature.substring(0, 8)}`;

      publicInterface.push({
        function_signature,
        raw_function_signature,
        functionName,
        parameters,
      });
    }
  }

  //check if hash collision exists
  let hashCollision = false;
  const seenSigs = new Map<String, number>();
  for (const sig of publicInterface) {
    if (seenSigs.has(sig.function_signature)) {
      hashCollision = true;
      break;
    }
    seenSigs.set(sig.function_signature, 1);
  }
  if (hashCollision) {
    throw new Error("hash collision");
  }

  const interfaceCode = generate_interface_code(publicInterface);

  // Read the file content
  const content = readFileSync(sourceFilePath, "utf8");

  const finalProgram = content + "\n" + interfaceCode;
  createTypeScriptFile(outFilePath, finalProgram, true);
}

//vibe coded
function createTypeScriptFile(
  filePath: string,
  content: string,
  overwrite: boolean = false
): void {
  try {
    if (existsSync(filePath) && !overwrite) {
      throw new Error(
        `File ${filePath} already exists. Set overwrite to true to replace it.`
      );
    }

    const dir = dirname(filePath);
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }

    let fileContent: string = content;

    writeFileSync(filePath, fileContent, "utf8");
    console.log(`TypeScript file created successfully: ${filePath}`);
  } catch (error) {
    console.error("Error creating TypeScript file:", error.message);
    throw error;
  }
}

function parseBool(buffer: Uint8Array, offset: u64): boolean {
  return buffer[offset] == 0;
}
function parseU64(buffer: Uint8Array, offset: u64) {
  let bytes = buffer.slice(offset, offset + 8);
  //bytes.
}

interface callDataJson {
  signature: string;
  payloadJson: string;
}
//example of how generated code looks
export const exampleCodeJson = {
  call(calldata: string) {
    const json: callDataJson = JSON.parse(calldata);
    const signature = json.signature;

    if (signature == "func1") {
      const payloadJson = JSON.parse(json.payloadJson);
      let postID = payloadJson.postID;
      let postContent = payloadJson.content;
    }
  },
};

function generate_interface_code(publicInterface: publicInterface[]) {
  let baseCode = `
  export const contractbindings = {

    makecall(calldata: string) {
          const json = JSON.parse(calldata);
          const signature = json.signature;
  `;

  let functionHandlingCode = "";
  publicInterface.forEach((publicInterface) => {
    functionHandlingCode += `
          if(signature == "${publicInterface.functionName}") {`;
    publicInterface.parameters.forEach((param) => {
      if (param.isArray) {
        functionHandlingCode += `
              let ${param.name} : ${param.arrayType}[] = json.${param.name};`;
      } else {
        functionHandlingCode += `
              let ${param.name} : ${param.typeText} = json.${param.name};`;
      }
    });
    let functionArgs = publicInterface.parameters
      .map((m) => m.name.toString())
      .join(", ");
    //publicInterface.parameters.forEach((param) => {
    // functionArgs += `${param.name}`;
    //});

    let functionCall = `
              ${publicInterface.functionName}(${functionArgs});`;
    functionHandlingCode += functionCall;

    functionHandlingCode += `
          }`;
  });
  baseCode += `${functionHandlingCode}
    return "";
    }
  }`;

  return baseCode;
}

function getParameterTypes(func: FunctionDeclaration): functionParameter[] {
  const parameters = func.getParameters();

  return parameters.map((param) => {
    const name = param.getName();
    const typeNode = param.getTypeNode();
    //const isArrayType = ArrayTypeNode;
    const isRestParameter = param.isRestParameter();
    const isOptionalParameter = param.isOptional();
    const isParameterProperty = param.isParameterProperty();
    const typeText = typeNode?.getText() || "any";

    const type = param.getType();
    const inferredType = type.getText();

    const isArray = type.isArray();
    //const arrayType = ParameterDeclaration.isArrayLiteralExpression(param);
    let arrayType = "";
    if (typeText.endsWith("[]")) {
      arrayType = typeText.slice(0, -2);
    }

    const isTuple = type.isTuple();

    return {
      name,
      typeText,
      inferredType,
      isRestParameter,
      isOptionalParameter,
      isParameterProperty,
      isArray,
      arrayType,
      isTuple,
    };
  });
}

/*
export const exampleCodeBytes = {
  call(calldata: Uint8Array) {
    const signature = calldata.slice(0, 4);
    if (signature == new Uint8Array([22, 33, 22, 33])) {
      //u64
      let arg1 = calldata.slice(4, 4 + 8);
      //string
      //let arg2_offset = calldata.slice(12, 12 + 4);
      //let arg2_length = calldata.slice(16, 16 + 4);
      let arg2_offset = 55;
      let arg2_length = 33;
      let arg2_string_bytes = calldata.slice(
        arg2_offset,
        arg2_offset + arg2_length
      );
    }
    //makeReply signature
    if (signature == new Uint8Array([99, 55, 22, 33])) {
    }
  },
};
function bytes_generate_interface_code(publicInterface: publicInterface[]) {
  const baseCode = `
    call(x: list<u8>) {
    x.slice()
  }
  `;

  publicInterface.forEach((publicInterface) => {
    const sigBytes = hexToUint8Array(publicInterface.function_signature);
    const var1_name = `____sig_${publicInterface.functionName}`;

    const code = `
      const ${var1_name} = new Uint8Array([${sigBytes[0]},${sigBytes[1]},${sigBytes[2]},${sigBytes[3]}]);
      if(${CALLDATA}. == ${var1_name})
    `;
  });
}
  */
