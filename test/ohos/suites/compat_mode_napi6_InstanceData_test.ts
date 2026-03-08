import "../runtime/console_shim";
import suite from "../src/compat-mode/napi6/InstanceData.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi6/InstanceData.test.ts", suite, "/tmp");
