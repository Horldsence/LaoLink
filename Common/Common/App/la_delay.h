/********************************** (C) COPYRIGHT  *******************************
 * File Name          : la_delay.h
 * Description        : FreeRTOS-safe busy-wait delay for the logic analyzer.
 *                      The V5F FreeRTOS port uses SysTick1 as the RTOS tick,
 *                      so the WCH Delay_Us/Delay_Ms (which drive SysTick1 on
 *                      V5F and stop it afterwards) cannot be used. These
 *                      routines use SysTick0 instead, which is unused on V5F.
 ********************************************************************************/
#ifndef __LA_DELAY_H
#define __LA_DELAY_H

#ifdef __cplusplus
extern "C"
{
#endif

#include "ch32h417.h"

void LA_Delay_Us(uint32_t n);
void LA_Delay_Ms(uint32_t n);

#ifdef __cplusplus
}
#endif

#endif
