import "../runtime/console_shim";
import suite from "../src/compat-mode/ArrayBuffer.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/ArrayBuffer.test.ts", suite, "/tmp");
