import abilityTest from './Ability.test';
import errorMessage from './napi/ErrorMessage.test';
import valueTest from './napi/value.test';
import generator from './napi/Genertor.test';
import objectAttrTest from './napi/ObjectAttr.test';

export default function testsuite() {
  abilityTest();
  errorMessage();
  valueTest();
  generator();
  objectAttrTest();
}