{
  inputs.hello.url = "abc";

  inputs.another_one = {
    url = "hello_world";
  };

  outputs = inputs@{hello   , ...}: {};
}
