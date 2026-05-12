import "../runtime/console_shim";
import suite from "../source/compat-mode/napi8/Object.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi8/Object.test.ts", suite, "/tmp");
