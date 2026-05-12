import "../runtime/console_shim";
import suite from "../source/compat-mode/JsValue.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/JsValue.test.ts", suite, "/tmp");
