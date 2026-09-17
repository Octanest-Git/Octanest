defmodule AppTest do
  use ExUnit.Case

  test "greet" do
    assert App.greet("Octanest") == "Hello, Octanest!"
  end
end
