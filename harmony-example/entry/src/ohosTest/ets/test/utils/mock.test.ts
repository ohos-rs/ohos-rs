import { MockKit } from "./framework.test";

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
    claser
  };
};
