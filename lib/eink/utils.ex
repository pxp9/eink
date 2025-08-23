defmodule EInk.Utils do
  use Rustler,
    otp_app: :eink,
    crate: :eink_utils,
    target: System.get_env("RUSTLER_TARGET")

  def nif_load(_path), do: :erlang.nif_error(:nif_not_loaded)
  def nif_save(_image, _path), do: :erlang.nif_error(:nif_not_loaded)
  def nif_to_binary(_image), do: :erlang.nif_error(:nif_not_loaded)
  def nif_resize(_image, _width, _height), do: :erlang.nif_error(:nif_not_loaded)
  def nif_dither_rgb(_image, _algorithm, _palette), do: :erlang.nif_error(:nif_not_loaded)
  def nif_dither_grayscale(_image, _algorithm, _bit_depth), do: :erlang.nif_error(:nif_not_loaded)
end
