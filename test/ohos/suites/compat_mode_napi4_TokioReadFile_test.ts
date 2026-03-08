import "../runtime/console_shim";
import suite from "../src/compat-mode/napi4/TokioReadFile.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("compat-mode/napi4/TokioReadFile.test.ts", suite, "/tmp");
