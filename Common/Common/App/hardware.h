/********************************** (C) COPYRIGHT  *******************************
* File Name          : hardware.h
* Description        : LaoLink logic analyzer hardware layer (CH32H417WEU6).
*                      Runs on the V5F core under FreeRTOS:
*                        - 8ch digital capture : UHSIF PORT16-23
*                          (PD10-PD15, PF0, PF1 = UFP0-UFP7), clk out PC0
*                        - 2ch analog capture  : HSADC PC2/PC3
*                        - upload              : USB3.0 (USBSS), USB2.0 (USBHS) fallback
*                        - VIO18 threshold     : DAC2/PA5
*                        - status LEDs         : PB10 (UFP_LED0/link), PB11 (UFP_LED1/run)
*                      Ported from the CH32H417_Logic_Analyzer_1v0 reference.
********************************************************************************/
#ifndef __HARDWARE_H
#define __HARDWARE_H

#ifdef __cplusplus
 extern "C" {
#endif

#include "ch32h417.h"
#include "debug.h"

/* UHSIF <-> USB buffer chain geometry (from the reference design):
 * 8 buffers per UHSIF thread, each DEF_USB_EP2_SS_SIZE*16 bytes. */
#define DEF_UHSIF_TXBUF_CNT  8
#define DEF_UHSIF_RXBUF_CNT  8

/*********************************************************************
 * Logic analyzer one-shot initialization. Call once on the V5F core
 * before the FreeRTOS scheduler is started.
 */
void LogicAnalyzer_Init(void);

/*********************************************************************
 * USB command processing (acquisition parameters, start/stop, IAP...).
 * Call repeatedly from a low-priority FreeRTOS task.
 */
void LogicAnalyzer_CmdProcess(void);

/*********************************************************************
 * High-rate data pumping (UHSIF -> USB, HSADC -> USB, overflow check).
 * Call repeatedly from a high-priority FreeRTOS task.
 */
void LogicAnalyzer_DataProcess(void);

#ifdef __cplusplus
}
#endif

#endif
