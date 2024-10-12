import abilityTest from './Ability.test';
import errorMessage from './napi/ErrorMessage.test';
import valueTest from './napi/Value.test';
import generator from './napi/Genertor.test';
import objectAttrTest from './napi/ObjectAttr.test';
import workerTest from './napi/Worker.test';
import strictTest from './napi/Stricts.test';

export default function testsuite() {
  abilityTest();
  errorMessage();
  valueTest();
  generator();
  objectAttrTest();
  strictTest();
  workerTest();
}