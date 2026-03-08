import "../runtime/console_shim";
import suite from "../src/napi/ObjectAttr.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("napi/ObjectAttr.test.ts", suite, "/tmp");
