library ieee;
use ieee.std_logic_1164.all;

library work;
use work.test_nspace.all;

entity my_0_test_0_space_0_link_comp_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    --  some doc 
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(53 downto 0);
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(53 downto 0);
    c_valid : in std_logic;
    c_ready : out std_logic;
    c_data : in std_logic_vector(53 downto 0);
    d_valid : out std_logic;
    d_ready : in std_logic;
    d_data : out std_logic_vector(53 downto 0)
  );
end my_0_test_0_space_0_link_comp_com;

architecture my_0_test_0_space_0_link_comp of my_0_test_0_space_0_link_comp_com is
begin
end my_0_test_0_space_0_link_comp;