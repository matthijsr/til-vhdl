library ieee;
use ieee.std_logic_1164.all;

library work;
use work.test_nspace.all;

entity my_0_test_0_space_0_high_complexity_fixed_generics_streamlet_com is
  generic (
    some_param : integer := 0;
    some_dim : positive := 4
  );
  port (
    clk : in std_logic;
    rst : in std_logic;
    x_valid : in std_logic;
    x_ready : out std_logic;
    x_data : in std_logic_vector(53 downto 0);
    x_last : in std_logic_vector(5 downto 0);
    x_stai : in std_logic;
    x_endi : in std_logic;
    x_strb : in std_logic_vector(1 downto 0);
    y_valid : out std_logic;
    y_ready : in std_logic;
    y_data : out std_logic_vector(53 downto 0);
    y_last : out std_logic_vector(5 downto 0);
    y_stai : out std_logic;
    y_endi : out std_logic;
    y_strb : out std_logic_vector(1 downto 0)
  );
end my_0_test_0_space_0_high_complexity_fixed_generics_streamlet_com;

architecture my_0_test_0_space_0_high_complexity_fixed_generics_streamlet of my_0_test_0_space_0_high_complexity_fixed_generics_streamlet_com is
begin
end my_0_test_0_space_0_high_complexity_fixed_generics_streamlet;