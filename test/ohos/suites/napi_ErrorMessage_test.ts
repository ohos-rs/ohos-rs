import "../runtime/console_shim";
import suite from "../src/napi/ErrorMessage.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("napi/ErrorMessage.test.ts", suite, "/tmp");
