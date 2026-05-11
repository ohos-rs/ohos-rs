import "../runtime/console_shim";
import suite from "../source/napi/ObjectAttr.test";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("napi/ObjectAttr.test.ts", suite, "/tmp");
