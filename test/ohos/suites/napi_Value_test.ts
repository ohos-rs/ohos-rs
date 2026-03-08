import "../runtime/console_shim";
import suite from "../src/napi/Value.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("napi/Value.test.ts", suite, "/tmp");
