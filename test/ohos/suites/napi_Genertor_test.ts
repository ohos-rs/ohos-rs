import "../runtime/console_shim";
import suite from "../src/napi/Genertor.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("napi/Genertor.test.ts", suite, "/tmp");
