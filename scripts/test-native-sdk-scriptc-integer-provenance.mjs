// Deterministic lowered-IR regression for the local ScriptC overlay. It proves
// that closed-world provenance is retained while overflow, external f64 input
// and address-taken callbacks continue to fail closed.
const compilerPath = process.argv[2];
if (compilerPath === undefined) {
  throw new Error("usage: node scripts/test-native-sdk-scriptc-integer-provenance.mjs <int-infer.js>");
}
const { checkLibraryIntegerSlots } = await import(compilerPath);

const loc = { file: "f9-probe.ts", start: 0, end: 1 };
const F64 = { kind: "f64" };
const BOOL = { kind: "bool" };
const VOID = { kind: "void" };
const MODEL = { kind: "record", shapeId: "model" };
const MSG = { kind: "record", shapeId: "msg" };
const TRANSIT = { kind: "record", shapeId: "transit" };
const EXTERNAL_MODEL = { kind: "record", shapeId: "external-model" };
const VALIDATED_MODEL = { kind: "record", shapeId: "validated-model" };
const MODELS = { kind: "array", elem: MODEL };
const CALLBACK = { kind: "func", params: [MODEL, F64], ret: VOID };

const variable = (localId, type) => ({ kind: "varRef", localId, type, loc });
const read = (obj, shapeId, field) => ({
  kind: "recordGet",
  obj,
  shapeId,
  field,
  type: F64,
  loc,
});
const numericLiteral = (value, spelling = String(value)) => ({ kind: "numLit", value, spelling, type: F64, loc });
const comparison = (op, left, right) => ({ kind: "bin", op, left, right, type: BOOL, loc });
const logical = (op, left, right) => ({ kind: "logical", op, left, right, type: BOOL, loc });
const isNotSafeInteger = (localId) => ({
  kind: "unary",
  op: "!",
  operand: { kind: "libCall", fn: "number.isSafeInteger", args: [variable(localId, F64)], type: BOOL, loc },
  type: BOOL,
  loc,
});
const invalidSuccessorGuard = (localId, limitGlobalId) => logical(
  "||",
  logical("||", isNotSafeInteger(localId), comparison("<=", variable(localId, F64), numericLiteral(0))),
  comparison(">=", variable(localId, F64), variable(limitGlobalId, F64)),
);
const incremented = (localId) => ({
  kind: "bin",
  op: "+",
  left: variable(localId, F64),
  right: numericLiteral(1),
  type: F64,
  loc,
});
const guardedSuccessorFunction = (name, localId, limitGlobalId) => ({
  name,
  params: [{ localId, name: "value", type: F64 }],
  returnType: F64,
  locals: [{ id: localId, name: "value", type: F64, mutable: false }],
  body: [{
    kind: "if",
    cond: invalidSuccessorGuard(localId, limitGlobalId),
    then: [{ kind: "return", value: numericLiteral(0), loc }],
    else_: null,
    loc,
  }, { kind: "return", value: incremented(localId), loc }],
  loc,
});

const module = {
  irVersion: 3,
  sourceFile: "f9-probe.ts",
  entry: "dispatch",
  lib: {
    profileName: "f9-probe",
    prefix: "f9_",
    initSymbol: "f9_init",
    sinkRegisterSymbol: "f9_sink",
    collectSymbol: null,
    resultResetSymbol: null,
    exports: [
      { symbol: "f9_unsafe", fnName: "unsafeExternal", params: ["f64"], returns: "void" },
      { symbol: "f9_safe", fnName: "safeExternal", params: ["f64"], returns: "void" },
      { symbol: "f9_next_guarded", fnName: "nextGuarded", params: ["f64"], returns: "f64" },
    ],
    trapOverlays: [],
  },
  records: [
    { id: "model", fields: [
      { name: "id", type: F64 },
      { name: "length", type: F64 },
      { name: "overflow", type: F64 },
      { name: "unsafe", type: F64 },
      { name: "viaRecord", type: F64 },
      { name: "callbackUnsafe", type: F64 },
      { name: "callbackIndex", type: F64 },
      { name: "callbackMixed", type: F64 },
      { name: "callbackGuarded", type: F64 },
      { name: "guardedWhole", type: F64 },
      { name: "deep", type: F64 },
      { name: "guardedSuccessor", type: F64 },
      { name: "mutableGuardedSuccessor", type: F64 },
      { name: "computedGuardedSuccessor", type: F64 },
      { name: "zeroInitializedGlobal", type: F64 },
      { name: "continueGuardedSuccessor", type: F64 },
    ] },
    { id: "msg", fields: [{ name: "id", type: F64 }] },
    { id: "transit", fields: [{ name: "id", type: F64 }] },
    { id: "external-model", fields: [{ name: "unsafe", type: F64 }, { name: "clamped", type: F64 }] },
    { id: "validated-model", fields: [{ name: "safe", type: F64 }] },
  ],
  globals: [
    { id: "%g.max-safe", name: "MAX_SAFE", type: F64, mutable: false },
    { id: "%g.mutable-max-safe", name: "mutableMaxSafe", type: F64, mutable: true },
    { id: "%g.computed-max-safe", name: "computedMaxSafe", type: F64, mutable: false },
    { id: "%g.zero-only", name: "zeroOnly", type: F64, mutable: false },
  ],
  functions: [
    {
      name: "dispatch",
      params: [
        { localId: "model.0", name: "model", type: MODEL },
        { localId: "msg.0", name: "msg", type: MSG },
      ],
      returnType: VOID,
      locals: [
        { id: "model.0", name: "model", type: MODEL, mutable: false },
        { id: "msg.0", name: "msg", type: MSG, mutable: false },
      ],
      body: [{
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "store",
          args: [
            variable("model.0", MODEL),
            read(variable("msg.0", MSG), "msg", "id"),
          ],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "recordSet",
        obj: variable("model.0", MODEL),
        shapeId: "model",
        field: "guardedSuccessor",
        value: {
          kind: "call",
          callee: "nextGuarded",
          args: [read(variable("msg.0", MSG), "msg", "id")],
          type: F64,
          loc,
        },
        loc,
      }, {
        kind: "recordSet",
        obj: variable("model.0", MODEL),
        shapeId: "model",
        field: "mutableGuardedSuccessor",
        value: {
          kind: "call",
          callee: "nextWithMutableLimit",
          args: [read(variable("msg.0", MSG), "msg", "id")],
          type: F64,
          loc,
        },
        loc,
      }, {
        kind: "recordSet",
        obj: variable("model.0", MODEL),
        shapeId: "model",
        field: "computedGuardedSuccessor",
        value: {
          kind: "call",
          callee: "nextWithComputedLimit",
          args: [read(variable("msg.0", MSG), "msg", "id")],
          type: F64,
          loc,
        },
        loc,
      }, {
        kind: "recordSet",
        obj: variable("model.0", MODEL),
        shapeId: "model",
        field: "zeroInitializedGlobal",
        value: { kind: "call", callee: "readZeroInitializedGlobal", args: [], type: F64, loc },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "closure",
          fnName: "callbackGuarded",
          captures: [],
          type: CALLBACK,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "storeContinueGuarded",
          args: [variable("model.0", MODEL), read(variable("msg.0", MSG), "msg", "id")],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "deep0",
          args: [variable("model.0", MODEL), read(variable("msg.0", MSG), "msg", "id")],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "%arr.map.0",
          args: [
            { kind: "arrayLit", elems: [], type: MODELS, loc },
            { kind: "closure", fnName: "callbackIndex", captures: [], type: CALLBACK, loc },
          ],
          type: MODELS,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "storeGuardedWhole",
          args: [variable("model.0", MODEL), read(variable("msg.0", MSG), "msg", "id")],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "closure",
          fnName: "callbackUnsafe",
          captures: [],
          type: CALLBACK,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "%arr.map.0",
          args: [
            { kind: "arrayLit", elems: [], type: MODELS, loc },
            { kind: "closure", fnName: "callbackMixed", captures: [], type: CALLBACK, loc },
          ],
          type: MODELS,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "callbackMixed",
          args: [
            variable("model.0", MODEL),
            read(variable("msg.0", MSG), "msg", "id"),
          ],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "storeOverflow",
          args: [
            variable("model.0", MODEL),
            read(variable("msg.0", MSG), "msg", "id"),
          ],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "consumeTransit",
          args: [
            variable("model.0", MODEL),
            {
              kind: "recordLit",
              fields: [{ name: "id", value: read(variable("msg.0", MSG), "msg", "id") }],
              type: TRANSIT,
              loc,
            },
          ],
          type: VOID,
          loc,
        },
        loc,
      }],
      loc,
    },
    {
      name: "%arr.map.0",
      params: [
        { localId: "hof.models", name: "models", type: MODELS },
        { localId: "hof.callback", name: "callback", type: CALLBACK },
      ],
      returnType: MODELS,
      locals: [
        { id: "hof.models", name: "models", type: MODELS, mutable: false },
        { id: "hof.callback", name: "callback", type: CALLBACK, mutable: false },
        { id: "hof.index", name: "index", type: F64, mutable: false },
      ],
      body: [{
        kind: "varDecl",
        localId: "hof.index",
        init: { kind: "numLit", value: 0, spelling: "0", type: F64, loc },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "callValue",
          callee: variable("hof.callback", CALLBACK),
          args: [
            { kind: "arrayGet", arr: variable("hof.models", MODELS), index: variable("hof.index", F64), type: MODEL, loc },
            variable("hof.index", F64),
          ],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "return",
        value: variable("hof.models", MODELS),
        loc,
      }],
      loc,
    },
    {
      name: "%init.integer",
      params: [],
      returnType: VOID,
      locals: [],
      body: [{
        kind: "assign",
        localId: "%g.max-safe",
        value: { kind: "numLit", value: 9007199254740991, spelling: "9007199254740991", type: F64, loc },
        loc,
      }, {
        kind: "assign",
        localId: "%g.mutable-max-safe",
        value: numericLiteral(9007199254740991),
        loc,
      }, {
        kind: "assign",
        localId: "%g.computed-max-safe",
        value: {
          kind: "bin",
          op: "+",
          left: numericLiteral(9007199254740991),
          right: numericLiteral(0),
          type: F64,
          loc,
        },
        loc,
      }],
      loc,
    },
    guardedSuccessorFunction("nextGuarded", "guarded.value", "%g.max-safe"),
    guardedSuccessorFunction("nextWithMutableLimit", "mutable-guarded.value", "%g.mutable-max-safe"),
    guardedSuccessorFunction("nextWithComputedLimit", "computed-guarded.value", "%g.computed-max-safe"),
    {
      name: "readZeroInitializedGlobal",
      params: [],
      returnType: F64,
      locals: [],
      body: [{ kind: "return", value: variable("%g.zero-only", F64), loc }],
      loc,
    },
    {
      name: "storeContinueGuarded",
      params: [
        { localId: "continue.model", name: "model", type: MODEL },
        { localId: "continue.value", name: "value", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "continue.model", name: "model", type: MODEL, mutable: false },
        { id: "continue.value", name: "value", type: F64, mutable: false },
        { id: "continue.row", name: "row", type: MODEL, mutable: false },
      ],
      body: [{
        kind: "forOf",
        localId: "continue.row",
        iterable: { kind: "arrayLit", elems: [], type: MODELS, loc },
        body: [{
          kind: "if",
          cond: invalidSuccessorGuard("continue.value", "%g.max-safe"),
          then: [{ kind: "continue", loc }],
          else_: null,
          loc,
        }, {
          kind: "recordSet",
          obj: variable("continue.model", MODEL),
          shapeId: "model",
          field: "continueGuardedSuccessor",
          value: incremented("continue.value"),
          loc,
        }],
        loc,
      }],
      loc,
    },
    {
      name: "callbackGuarded",
      params: [
        { localId: "callback.guarded.model", name: "model", type: MODEL },
        { localId: "callback.guarded.value", name: "value", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "callback.guarded.model", name: "model", type: MODEL, mutable: false },
        { id: "callback.guarded.value", name: "value", type: F64, mutable: false },
      ],
      body: [{
        kind: "if",
        cond: invalidSuccessorGuard("callback.guarded.value", "%g.max-safe"),
        then: [{ kind: "return", value: null, loc }],
        else_: null,
        loc,
      }, {
        kind: "recordSet",
        obj: variable("callback.guarded.model", MODEL),
        shapeId: "model",
        field: "callbackGuarded",
        value: incremented("callback.guarded.value"),
        loc,
      }],
      loc,
    },
    {
      name: "callbackIndex",
      params: [
        { localId: "callback.index.model", name: "model", type: MODEL },
        { localId: "callback.index.value", name: "index", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "callback.index.model", name: "model", type: MODEL, mutable: false },
        { id: "callback.index.value", name: "index", type: F64, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: variable("callback.index.model", MODEL),
        shapeId: "model",
        field: "callbackIndex",
        value: variable("callback.index.value", F64),
        loc,
      }],
      loc,
    },
    {
      name: "safeExternal",
      params: [{ localId: "validated.0", name: "validated", type: F64 }],
      returnType: VOID,
      locals: [{ id: "validated.0", name: "validated", type: F64, mutable: false }],
      body: [{
        kind: "if",
        cond: {
          kind: "logical",
          op: "&&",
          left: {
            kind: "libCall",
            fn: "number.isSafeInteger",
            args: [variable("validated.0", F64)],
            type: { kind: "bool" },
            loc,
          },
          right: {
            kind: "bin",
            op: "<=",
            left: variable("validated.0", F64),
            right: { kind: "numLit", value: 100, spelling: "100", type: F64, loc },
            type: { kind: "bool" },
            loc,
          },
          type: { kind: "bool" },
          loc,
        },
        then: [{
          kind: "exprStmt",
          expr: {
            kind: "recordLit",
            fields: [{ name: "safe", value: variable("validated.0", F64) }],
            type: VALIDATED_MODEL,
            loc,
          },
          loc,
        }],
        else_: null,
        loc,
      }],
      loc,
    },
    {
      name: "storeGuardedWhole",
      params: [
        { localId: "guarded.whole.model", name: "model", type: MODEL },
        { localId: "guarded.whole.value", name: "value", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "guarded.whole.model", name: "model", type: MODEL, mutable: false },
        { id: "guarded.whole.value", name: "value", type: F64, mutable: false },
        { id: "guarded.whole.result", name: "whole", type: F64, mutable: false },
      ],
      body: [{
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "callbackMixed",
          args: [variable("guarded.whole.model", MODEL), variable("guarded.whole.value", F64)],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "if",
        cond: logical(
          "||",
          isNotSafeInteger("guarded.whole.value"),
          {
            kind: "unary",
            op: "!",
            operand: comparison(">", variable("guarded.whole.value", F64), numericLiteral(0)),
            type: BOOL,
            loc,
          },
        ),
        then: [{ kind: "return", value: null, loc }],
        else_: null,
        loc,
      }, {
        kind: "varDecl",
        localId: "guarded.whole.result",
        init: {
          kind: "libCall",
          fn: "math.trunc",
          args: [variable("guarded.whole.value", F64)],
          type: F64,
          loc,
        },
        loc,
      }, {
        kind: "if",
        cond: logical(
          "||",
          {
            kind: "unary",
            op: "!",
            operand: comparison(">", variable("guarded.whole.result", F64), numericLiteral(0)),
            type: BOOL,
            loc,
          },
          comparison(">", variable("guarded.whole.result", F64), numericLiteral(9007199254740991)),
        ),
        then: [{ kind: "return", value: null, loc }],
        else_: null,
        loc,
      }, {
        kind: "recordSet",
        obj: variable("guarded.whole.model", MODEL),
        shapeId: "model",
        field: "guardedWhole",
        value: {
          kind: "libCall",
          fn: "math.trunc",
          args: [variable("guarded.whole.result", F64)],
          type: F64,
          loc,
        },
        loc,
      }],
      loc,
    },
    {
      name: "callbackMixed",
      params: [
        { localId: "callback.mixed.model", name: "model", type: MODEL },
        { localId: "callback.mixed.value", name: "index", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "callback.mixed.model", name: "model", type: MODEL, mutable: false },
        { id: "callback.mixed.value", name: "index", type: F64, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: variable("callback.mixed.model", MODEL),
        shapeId: "model",
        field: "callbackMixed",
        value: variable("callback.mixed.value", F64),
        loc,
      }],
      loc,
    },
    {
      name: "callbackUnsafe",
      params: [
        { localId: "callback.model", name: "model", type: MODEL },
        { localId: "callback.0", name: "value", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "callback.model", name: "model", type: MODEL, mutable: false },
        { id: "callback.0", name: "value", type: F64, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: variable("callback.model", MODEL),
        shapeId: "model",
        field: "callbackUnsafe",
        value: variable("callback.0", F64),
        loc,
      }],
      loc,
    },
    {
      name: "consumeTransit",
      params: [
        { localId: "model.transit", name: "model", type: MODEL },
        { localId: "transit.0", name: "transit", type: TRANSIT },
      ],
      returnType: VOID,
      locals: [
        { id: "model.transit", name: "model", type: MODEL, mutable: false },
        { id: "transit.0", name: "transit", type: TRANSIT, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: variable("model.transit", MODEL),
        shapeId: "model",
        field: "viaRecord",
        value: read(variable("transit.0", TRANSIT), "transit", "id"),
        loc,
      }],
      loc,
    },
    {
      name: "storeOverflow",
      params: [
        { localId: "model.overflow", name: "model", type: MODEL },
        { localId: "id.overflow", name: "id", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "model.overflow", name: "model", type: MODEL, mutable: false },
        { id: "id.overflow", name: "id", type: F64, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: variable("model.overflow", MODEL),
        shapeId: "model",
        field: "overflow",
        value: {
          kind: "bin",
          op: "+",
          left: variable("id.overflow", F64),
          right: { kind: "numLit", value: 1, spelling: "1", type: F64, loc },
          type: F64,
          loc,
        },
        loc,
      }],
      loc,
    },
    {
      name: "store",
      params: [
        { localId: "model.1", name: "model", type: MODEL },
        { localId: "id.0", name: "id", type: F64 },
      ],
      returnType: VOID,
      locals: [
        { id: "model.1", name: "model", type: MODEL, mutable: false },
        { id: "id.0", name: "id", type: F64, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: variable("model.1", MODEL),
        shapeId: "model",
        field: "id",
        value: variable("id.0", F64),
        loc,
      }],
      loc,
    },
    {
      name: "storeLength",
      params: [
        { localId: "model.2", name: "model", type: MODEL },
        { localId: "models.0", name: "models", type: MODELS },
      ],
      returnType: VOID,
      locals: [
        { id: "model.2", name: "model", type: MODEL, mutable: false },
        { id: "models.0", name: "models", type: MODELS, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: variable("model.2", MODEL),
        shapeId: "model",
        field: "length",
        value: {
          kind: "arrIntrinsic",
          method: "length",
          receiver: variable("models.0", MODELS),
          args: [],
          type: F64,
          loc,
        },
        loc,
      }],
      loc,
    },
    {
      name: "unsafeExternal",
      params: [{ localId: "external.0", name: "external", type: F64 }],
      returnType: VOID,
      locals: [
        { id: "external.0", name: "external", type: F64, mutable: false },
        { id: "clamped.0", name: "clamped", type: F64, mutable: true },
      ],
      body: [{
        kind: "exprStmt",
        expr: {
          kind: "call",
          callee: "callbackMixed",
          args: [
            { kind: "recordLit", fields: [], type: MODEL, loc },
            variable("external.0", F64),
          ],
          type: VOID,
          loc,
        },
        loc,
      }, {
        kind: "exprStmt",
        expr: {
          kind: "recordLit",
          fields: [{ name: "unsafe", value: variable("external.0", F64) }],
          type: EXTERNAL_MODEL,
          loc,
        },
        loc,
      }, {
        kind: "varDecl",
        localId: "clamped.0",
        init: variable("external.0", F64),
        loc,
      }, {
        kind: "if",
        cond: { kind: "bin", op: "<", left: variable("clamped.0", F64), right: { kind: "numLit", value: 0, spelling: "0", type: F64, loc }, type: { kind: "bool" }, loc },
        then: [{ kind: "assign", localId: "clamped.0", value: { kind: "numLit", value: 0, spelling: "0", type: F64, loc }, loc }],
        else_: null,
        loc,
      }, {
        kind: "if",
        cond: { kind: "bin", op: ">", left: variable("clamped.0", F64), right: { kind: "numLit", value: 1024, spelling: "1024", type: F64, loc }, type: { kind: "bool" }, loc },
        then: [{ kind: "assign", localId: "clamped.0", value: { kind: "numLit", value: 1024, spelling: "1024", type: F64, loc }, loc }],
        else_: null,
        loc,
      }, {
        kind: "exprStmt",
        expr: { kind: "recordLit", fields: [{ name: "clamped", value: variable("clamped.0", F64) }], type: EXTERNAL_MODEL, loc },
        loc,
      }],
      loc,
    },
  ],
};

// A source-order-independent chain longer than the previous 64 global passes.
// The worklist must propagate the declared Msg integer without accepting a
// partial under-approximation or falling back to TOP.
for (let index = 0; index < 80; index += 1) {
  const final = index === 79;
  module.functions.push({
    name: `deep${index}`,
    params: [
      { localId: `deep.model.${index}`, name: "model", type: MODEL },
      { localId: `deep.value.${index}`, name: "value", type: F64 },
    ],
    returnType: VOID,
    locals: [
      { id: `deep.model.${index}`, name: "model", type: MODEL, mutable: false },
      { id: `deep.value.${index}`, name: "value", type: F64, mutable: false },
    ],
    body: final ? [{
      kind: "recordSet",
      obj: variable(`deep.model.${index}`, MODEL),
      shapeId: "model",
      field: "deep",
      value: variable(`deep.value.${index}`, F64),
      loc,
    }] : [{
      kind: "exprStmt",
      expr: {
        kind: "call",
        callee: `deep${index + 1}`,
        args: [variable(`deep.model.${index}`, MODEL), variable(`deep.value.${index}`, F64)],
        type: VOID,
        loc,
      },
      loc,
    }],
    loc,
  });
}

const config = {
  fns: new Map(),
  records: new Map([
    ["model", new Map([
      ["id", { cls: "i64", paths: ["Model.id"] }],
      ["length", { cls: "i64", paths: ["Model.length"] }],
      ["overflow", { cls: "i64", paths: ["Model.overflow"] }],
      ["callbackUnsafe", { cls: "i64", paths: ["Model.callbackUnsafe"] }],
      ["callbackIndex", { cls: "i64", paths: ["Model.callbackIndex"] }],
      ["callbackMixed", { cls: "i64", paths: ["Model.callbackMixed"] }],
      ["callbackGuarded", { cls: "i64", paths: ["Model.callbackGuarded"] }],
      ["guardedWhole", { cls: "i64", paths: ["Model.guardedWhole"] }],
      ["deep", { cls: "i64", paths: ["Model.deep"] }],
      ["guardedSuccessor", { cls: "i64", paths: ["Model.guardedSuccessor"] }],
      ["mutableGuardedSuccessor", { cls: "i64", paths: ["Model.mutableGuardedSuccessor"] }],
      ["computedGuardedSuccessor", { cls: "i64", paths: ["Model.computedGuardedSuccessor"] }],
      ["zeroInitializedGlobal", { cls: "i64", paths: ["Model.zeroInitializedGlobal"] }],
      ["continueGuardedSuccessor", { cls: "i64", paths: ["Model.continueGuardedSuccessor"] }],
      ["viaRecord", { cls: "i64", paths: ["Model.viaRecord"] }],
    ])],
    ["msg", new Map([["id", { cls: "i64", paths: ["Msg_set.id"] }]])],
    ["external-model", new Map([
      ["unsafe", { cls: "i64", paths: ["Model.unsafe"] }],
      ["clamped", { cls: "i64", paths: ["Model.clamped"] }],
    ])],
    ["validated-model", new Map([["safe", { cls: "i64", paths: ["Model.safe"] }]])],
  ]),
};

const verdicts = checkLibraryIntegerSlots(module, config);
const outcomes = new Map(verdicts.map((verdict) => [verdict.path, `${verdict.outcome}:${verdict.obligation ?? "proved"}`]));
const expected = new Map([
  ["Model.safe", "prove:proved"],
  ["Model.viaRecord", "prove:proved"],
  ["Model.id", "prove:proved"],
  ["Model.length", "prove:proved"],
  ["Model.callbackUnsafe", "refuse:wholeness"],
  ["Model.callbackIndex", "prove:proved"],
  ["Model.callbackMixed", "refuse:wholeness"],
  ["Model.callbackGuarded", "prove:proved"],
  ["Model.guardedWhole", "prove:proved"],
  ["Model.deep", "prove:proved"],
  ["Model.guardedSuccessor", "prove:proved"],
  ["Model.mutableGuardedSuccessor", "refuse:range"],
  ["Model.computedGuardedSuccessor", "refuse:range"],
  ["Model.zeroInitializedGlobal", "prove:proved"],
  ["Model.continueGuardedSuccessor", "prove:proved"],
  ["Model.clamped", "refuse:wholeness"],
  ["Model.overflow", "refuse:range"],
  ["Model.unsafe", "refuse:wholeness"],
]);
if (outcomes.size !== expected.size) {
  throw new Error(`unexpected verdict count: ${outcomes.size}`);
}
for (const [path, expectedOutcome] of expected) {
  const actualOutcome = outcomes.get(path);
  if (actualOutcome !== expectedOutcome) {
    throw new Error(`${path}: expected ${expectedOutcome}, got ${actualOutcome ?? "missing"}`);
  }
}
const guardedSuccessorVerdict = verdicts.find((verdict) => verdict.path === "Model.guardedSuccessor");
if (guardedSuccessorVerdict?.provenLo !== 0 || guardedSuccessorVerdict.provenHi !== 9007199254740991) {
  throw new Error("immutable global successor must retain the zero-init and literal upper bound");
}
const callbackIndexVerdict = verdicts.find((verdict) => verdict.path === "Model.callbackIndex");
if (callbackIndexVerdict?.provenLo !== 0 || callbackIndexVerdict.provenHi !== 4294967295) {
  throw new Error("trusted Array HOF callback index must retain its exact u32 range");
}
const zeroInitializedVerdict = verdicts.find((verdict) => verdict.path === "Model.zeroInitializedGlobal");
if (zeroInitializedVerdict?.provenLo !== 0 || zeroInitializedVerdict.provenHi !== 0) {
  throw new Error("unwritten immutable global must retain the zero-initialized range");
}

// An address-taken closure is dynamically callable even when the lowered
// module also constructs a precise literal of the same record shape. Its
// composite parameters must therefore taint structural field summaries.
const compositeLoc = { file: "address-taken-composite.ts", start: 0, end: 1 };
const CALLBACK_INPUT = { kind: "record", shapeId: "callback-input" };
const CALLBACK_OUTPUT = { kind: "record", shapeId: "callback-output" };
const COMPOSITE_CALLBACK = { kind: "func", params: [CALLBACK_INPUT, CALLBACK_OUTPUT], ret: VOID };
const compositeVariable = (localId, type) => ({ kind: "varRef", localId, type, loc: compositeLoc });
const addressTakenCompositeModule = {
  irVersion: 3,
  sourceFile: "address-taken-composite.ts",
  entry: "addressTakenCompositeEntry",
  records: [
    { id: "callback-input", fields: [{ name: "id", type: F64 }] },
    { id: "callback-output", fields: [{ name: "id", type: F64 }] },
  ],
  functions: [
    {
      name: "addressTakenCompositeEntry",
      params: [],
      returnType: VOID,
      locals: [],
      body: [
        {
          kind: "exprStmt",
          // This internal literal must not become the dynamic callback's input contract.
          expr: {
            kind: "recordLit",
            fields: [{
              name: "id",
              value: { kind: "numLit", value: 1, spelling: "1", type: F64, loc: compositeLoc },
            }],
            type: CALLBACK_INPUT,
            loc: compositeLoc,
          },
          loc: compositeLoc,
        },
        {
          kind: "exprStmt",
          expr: {
            kind: "closure",
            fnName: "addressTakenCompositeCallback",
            captures: [],
            type: COMPOSITE_CALLBACK,
            loc: compositeLoc,
          },
          loc: compositeLoc,
        },
      ],
      loc: compositeLoc,
    },
    {
      name: "addressTakenCompositeCallback",
      params: [
        { localId: "composite.input", name: "input", type: CALLBACK_INPUT },
        { localId: "composite.output", name: "output", type: CALLBACK_OUTPUT },
      ],
      returnType: VOID,
      locals: [
        { id: "composite.input", name: "input", type: CALLBACK_INPUT, mutable: false },
        { id: "composite.output", name: "output", type: CALLBACK_OUTPUT, mutable: false },
      ],
      body: [{
        kind: "recordSet",
        obj: compositeVariable("composite.output", CALLBACK_OUTPUT),
        shapeId: "callback-output",
        field: "id",
        value: {
          kind: "recordGet",
          obj: compositeVariable("composite.input", CALLBACK_INPUT),
          shapeId: "callback-input",
          field: "id",
          type: F64,
          loc: compositeLoc,
        },
        loc: compositeLoc,
      }],
      loc: compositeLoc,
    },
  ],
};
const compositeConfig = {
  fns: new Map(),
  records: new Map([
    ["callback-output", new Map([["id", { cls: "i64", paths: ["CallbackOutput.id"] }]])],
  ]),
};
const compositeVerdicts = checkLibraryIntegerSlots(addressTakenCompositeModule, compositeConfig);
const compositeVerdict = compositeVerdicts.find((verdict) => verdict.path === "CallbackOutput.id");
if (compositeVerdict?.outcome !== "refuse" || compositeVerdict.obligation !== "wholeness") {
  throw new Error(
    `address-taken composite must fail closed; got ${JSON.stringify(compositeVerdict)}`,
  );
}
console.log("scriptc integer provenance: PASS");
