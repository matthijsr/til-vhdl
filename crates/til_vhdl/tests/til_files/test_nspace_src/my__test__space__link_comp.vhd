library ieee;
use ieee.std_logic_1164.all;

library work;
use work.proj.all;

entity my__test__space__link_comp_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    --  some doc 
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(53 downto 0);
    c_valid : in std_logic;
    c_ready : out std_logic;
    c_data : in std_logic_vector(53 downto 0);
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(53 downto 0);
    d_valid : out std_logic;
    d_ready : in std_logic;
    d_data : out std_logic_vector(53 downto 0)
  );
end my__test__space__link_comp_com;

architecture my__test__space__link_comp of my__test__space__link_comp_com is
begin
  a_ready <= '1';
end my__test__space__link_comp;