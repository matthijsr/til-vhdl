library ieee;
use ieee.std_logic_1164.all;

library work;
use work.proj.all;

entity my__example__space__comp5_com is
  port (
    clk : in std_logic;
    rst : in std_logic;
    a_valid : in std_logic;
    a_ready : out std_logic;
    a_data : in std_logic_vector(53 downto 0);
    b_valid : out std_logic;
    b_ready : in std_logic;
    b_data : out std_logic_vector(53 downto 0);
    --  port documentation 
    c_valid : in std_logic;
    c_ready : out std_logic;
    c_data : in std_logic_vector(53 downto 0);
    d_valid : out std_logic;
    d_ready : in std_logic;
    d_data : out std_logic_vector(53 downto 0)
  );
end my__example__space__comp5_com;

architecture my__example__space__comp5 of my__example__space__comp5_com is
begin
end my__example__space__comp5;