import { MockKit } from '@ohos/hypium'

class MockName {
  test() {}
}

export const spy = () => {
  const mocker = new MockKit();

  const claser = new MockName();

  let mockfunc: Function = mocker.mockFunc(claser, claser.test);

  return {
    func: mockfunc,
    mocker,
  }
}