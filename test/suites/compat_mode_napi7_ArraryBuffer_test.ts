import "../runtime/console_shim";
import suite from "../source/compat-mode/napi7/ArraryBuffer.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi7/ArraryBuffer.test.ts", suite, "/tmp");
