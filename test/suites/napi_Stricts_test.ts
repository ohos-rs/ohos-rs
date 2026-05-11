import "../runtime/console_shim";
import suite from "../source/napi/Stricts.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("napi/Stricts.test.ts", suite, "/tmp");
