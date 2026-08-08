/********************************** (C) COPYRIGHT  *******************************
 * File Name          : la_delay.c
 * Description        : FreeRTOS-safe busy-wait delay for the logic analyzer.
 *                      Uses SysTick0 (unused by the V5F FreeRTOS port, which
 *                      runs its tick on SysTick1). Safe to call from both task
 *                      and interrupt context. Delay length follows the current
 *                      HCLKClock, so it stays correct after the sample-rate
 *                      PLL switching done by UHSIF_Clock_Set().
 ********************************************************************************/
#include "la_delay.h"

/*********************************************************************
 * @fn      LA_Delay_Us
 *
 * @brief   Microsecond busy-wait delay based on SysTick0.
 *
 * @param   n - Microseconds.
 *
 * @return  none
 */
void LA_Delay_Us(uint32_t n)
{
    uint32_t i = n * (HCLKClock / 1000000);

    SysTick0->ISR &= ~(1 << 0);
    SysTick0->CNT = 0;
    SysTick0->CMP = i;
    SysTick0->CTLR = (1 << 2);
    SysTick0->CTLR |= (1 << 0);

    while((SysTick0->ISR & (1 << 0)) != (1 << 0))
        ;
    SysTick0->CTLR &= ~(1 << 0);
}

/*********************************************************************
 * @fn      LA_Delay_Ms
 *
 * @brief   Millisecond busy-wait delay based on SysTick0.
 *
 * @param   n - Milliseconds.
 *
 * @return  none
 */
void LA_Delay_Ms(uint32_t n)
{
    uint32_t i = n * (HCLKClock / 1000);

    SysTick0->ISR &= ~(1 << 0);
    SysTick0->CNT = 0;
    SysTick0->CMP = i;
    SysTick0->CTLR = (1 << 2);
    SysTick0->CTLR |= (1 << 0);

    while((SysTick0->ISR & (1 << 0)) != (1 << 0))
        ;
    SysTick0->CTLR &= ~(1 << 0);
}
