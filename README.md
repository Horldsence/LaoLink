# LaoLink 嵌入式部分

## 文件结构

/--
  |-V3F CH32H417 Qingke V3F 内核源码
  |-V5F CH32H417 Qingke V5F 内核源码

## 软件结构

freeRTOS

## 功能规划

### CH32H417WEU6 (LaoLink)

数控电源输入输出    (GPIO IO 1pin 控制电源开关)
数控电源输出控制    (IIC 控制)
数字电流表电压表    (IIC 读取)
简易示波器         (ADC)
8通道逻辑分析仪     (8 GPIO HS IO 8pin)
屏幕              (SPI)
IIC扫描           (I3C 端口 扫描 2pin)

^
| 通过USART连接
v


### CH32V208GBU6 (WCHLinkW)

下载器与无线下载器
串口助手与无线串口助手

## 引脚分配
### CH32H417WEU6
```C
USB3.0:
SSTXB   -
SSTXA   -
SSRXB   -
SSRXA   -

SWD:
PB8     SWCLK
PB9     SWDIO

USART:
PC6     USART4_TX
PC7     USART4_RX

I3C:
PE15    I3C_SDA
PE14    I3C_SCL

I2C:
PC0     I2C2_SCL
PC1     I2C2_SDA
//
// 如果电源和电压共用I2C下更换USART
// 我们理论上可以做出来16通道的的逻辑分析仪
// 但是需要给I2C做互斥锁
//
// PA13    I2C3_SDA
// PA14    I2C3_SCL

SPI:
PD7     SPI1_MOSI
PF3     SPI1_MISO

GPIO:
PB0     POWER_CTRL
PC2     ADC1-UHSIF
PC3     ADC0-UHSIF
//
// 另外8路逻辑分析仪
//
// PA15
// PA14
// PA13
// PC9
// PC8
// PC7
// PC6
// PF2
PF1     UFP7
PF0     UFP6
PD15    UFP5
PD14    UFP4
PD13    UFP3
PD12    UFP2
PD11    UFP1
PD10    UFP0
```
### CH32V208GBU6
```C
SWD:
PA14    SWCLK
PA13    SWDIO

USB:
PB6     USB_D-
PB7     USB_D+

USART:
PA2     UART2_TX
PA3     UART2_RX

GPIO
PA4     MODES
PC9     LED_CON
PC7     LED_MODE
PA6     TDO
PA7     TDI
```
