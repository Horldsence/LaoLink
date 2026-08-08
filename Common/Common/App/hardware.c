/********************************** (C) COPYRIGHT  *******************************
* File Name          : hardware.c
* Description        : LaoLink logic analyzer hardware layer (CH32H417WEU6).
*                      V5F core only. The bare-metal main loop of the reference
*                      design is split into an init function plus two process
*                      functions that are driven by FreeRTOS tasks.
*                      Difference from the reference: SWD (PB8/PB9) is kept
*                      enabled on this board, so GPIO_Remap_SWJ_Disable is NOT
*                      called here.
********************************************************************************/
#include "hardware.h"

/* The logic analyzer runs on the V5F core only. The shared Common
 * folder is also compiled by the V3F project, so guard the body. */
#ifdef Core_V5F

#include "ch32h417_usbss_device.h"
#include "ch32h417_usbhs_device.h"
#include "ch32h417_logic.h"
#include "la_delay.h"

/*********************************************************************
 * @fn      VDDI8_IO_Change
 *
 * @brief   Turn off the internal 1.8v power supply (VIO18 supplied by the
 *          external DCDC, DAC2/PA5 sets the logic threshold voltage).
 *
 * @param   none
 *
 * @return  none
 */
void VDDI8_IO_Change(void)
{
    RCC_HB1PeriphClockCmd(RCC_HB1Periph_PWR, ENABLE);
    PWR->CTLR |= PWR_CTLR_VIO_SWCR;
    PWR->CTLR &= ~PWR_CTLR_VSEL_VIO18;
    PWR->CTLR = PWR->CTLR | PWR_CTLR_VSEL_VIO18_MODE5;
    Dac_Init(); /* Default output 3.3V power supply */
}

/*********************************************************************
 * @fn      LogicAnalyzer_Init
 *
 * @brief   Logic analyzer one-shot initialization.
 *
 * @param   none
 *
 * @return  none
 */
void LogicAnalyzer_Init(void)
{
    /* NOTE: unlike the reference design, SWD (PB8/PB9) is kept enabled. */

    /* Enable the IO function of PC6 pin (reserved for USART4_TX) */
    RCC_HB1PeriphClockCmd(RCC_HB1Periph_SWPMI, ENABLE);
    SWPMI->OR |= SWPMI_SWP_TBYP;

    /* Parameter initialization (also inits UFP_LED0/UFP_LED1 GPIOs) */
    LOGIC_Para_Init();
    Led_Time_Init();
    LA_Delay_Ms(100);
    VDDI8_IO_Change();                                                          /* Turn off VDDIO-1.8V power supply */
    Get_Flash_Size();                                                           /* Get FLASH size for IAP upgrade */
    RCC_HB2PeriphClockCmd(RCC_HB2Periph_GPIOF | RCC_HB2Periph_HSADC, ENABLE);   /* Enable ADC clock */
    RCC_HSADCCLKConfig(RCC_HSADCSource_PLLCLK);                                 /* Enable ADC PLL */
    /* USB initialization */
    USB_Timer_Init();
    USBSS_Device_Init(ENABLE);
}

/*********************************************************************
 * @fn      LogicAnalyzer_CmdProcess
 *
 * @brief   USB command processing, call repeatedly from a task.
 *
 * @param   none
 *
 * @return  none
 */
void LogicAnalyzer_CmdProcess(void)
{
    LOGIC_ADC_CMD_Process();    /* Logic analyzer and ADC acquisition parameter setting */
    IAP_JUMP_Process();         /* IAP processing */
}

/*********************************************************************
 * @fn      LogicAnalyzer_DataProcess
 *
 * @brief   High-rate data pumping, call repeatedly from a task.
 *
 * @param   none
 *
 * @return  none
 */
void LogicAnalyzer_DataProcess(void)
{
    LOGIC_UHSIF_Main_Process(); /* Logic analyzer data transfer */
    LOGIC_ADC_Main_Process();   /* ADC data acquisition */
    LOGIC_ADC_RAMOVER_Process();/* Report buffer overflow */
}

#endif /* Core_V5F */
