# LaoLink 嵌入式部分

## 文件结构
```
/--
  |-Common/Common        双核共享代码
  |  |-App/              应用层: hardware(硬件适配入口) + la_delay(RTOS安全延时)
  |  |-LogicAnalyzer/    分析仪核心: 命令处理/数据泵 + UHSIF 配置与预编译库(.o)
  |  |-USB/              USB 协议栈: USB3.0(USBSS) / USB2.0(USBHS) / 描述符
  |  |-shared.h          核间共享数据
  |-V3F CH32H417 Qingke V3F 内核源码 (User/main.c: 唤醒V5F + 心跳任务)
  |-V5F CH32H417 Qingke V5F 内核源码 (User/main.c + la_tasks.c: 分析仪任务)
```
## 软件结构

freeRTOS

## 逻辑分析仪固件说明 (V5F 内核)

基于 [CH32H417_Logic_Analyzer_Mini](../CH32H417_Logic_Analyzer_Mini) 参考工程移植,
上位机兼容 U3LogicAnalyzer / PulseView (USB3.0 直连, 兼容 USB2.0)。

```
V3F (0x00000000, 64K): 唤醒 V5F + 心跳任务 (后续电源/屏幕/UI 功能)
V5F (0x00010000, 128K): FreeRTOS 任务包装的逻辑分析仪
  |- la_data_task  UHSIF/HSADC -> USB 高速数据搬运
  |- la_cmd_task   USB 命令处理 (参数/启停/IAP)
```

- 8 通道数字采集: UHSIF PORT16-23 = PD10~PD15/PF0/PF1 (UFP0~UFP7), 时钟脚 PC0
- 2 通道模拟采集: HSADC PC2/PC3; VIO18 阈值调压: DAC2/PA5
- 状态灯: PB10 = UFP_LED0 (USB 连接), PB11 = UFP_LED1 (采集闪烁)
- 调试 printf: PB4 (USART8); SWD 保持可用 (PB8/PB9)
- UHSIF 底层库为预编译 `Common/Common/LogicAnalyzer/ch32h417_uhsif_it.o` (仅链接进 V5F)
- 采样 DMA 缓冲区 (256K) 位于 SRAM 0x20130000-0x20170000 (V5F 链接脚本 `.da1` 段)
- 注意: V5F 的 FreeRTOS 心跳在 SysTick1, 分析仪延时一律用 `la_delay.c` (SysTick0 忙等),
  不要再调用 `Delay_Us/Delay_Ms`; 切换采样率会改 PLL, RTOS 延时会有比例漂移 (已知, 可接受)


## 功能规划
```
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

CH32V208GBU6 (WCHLinkW)

下载器与无线下载器
串口助手与无线串口助手
```
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
PA14    I2C3_SCL
PA13    I2C3_SDA

SPI:
PD7     SPI1_MOSI
PF3     SPI1_MISO

GPIO:
PB0     POWER_CTRL
PC2     ADC1-UHSIF
PC3     ADC0-UHSIF
PC0     UHSIF_CLK_1
PB10    UFP_LED0
PB11    UFP_LED1
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
