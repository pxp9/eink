defmodule EinkTest do
  use ExUnit.Case
  doctest EInk

  test "EInk module exists" do
    assert Code.ensure_loaded?(EInk)
  end

  test "EInk.Driver behaviour exists" do
    assert Code.ensure_loaded?(EInk.Driver)
  end

  test "UC8276 driver exists and has capabilities" do
    assert Code.ensure_loaded?(EInk.Driver.UC8276)

    capabilities = EInk.Driver.UC8276.capabilities()
    assert capabilities.width == 400
    assert capabilities.height == 300
    assert capabilities.palette == :bw
    assert capabilities.partial_refresh == true
  end

  test "UC8179 driver exists and has capabilities" do
    assert Code.ensure_loaded?(EInk.Driver.UC8179)

    capabilities = EInk.Driver.UC8179.capabilities()
    assert capabilities.width == 800
    assert capabilities.height == 600
    assert capabilities.palette == :bw
    assert capabilities.partial_refresh == true
  end
end
