import "../runtime/console_shim";
import suite from "../source/napi/Worker.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("napi/Worker.test.ts", suite, "/tmp");
