/********************************** (C) COPYRIGHT *******************************
 * File Name          : main.c
 * Description        : Main program body for V5F.
 *                      LaoLink logic analyzer (CH32H417WEU6):
 *                      the acquisition/USB engine of the reference
 *                      CH32H417_Logic_Analyzer design runs on this core,
 *                      wrapped in FreeRTOS tasks (see la_tasks.c).
 *********************************************************************************
 * Copyright (c) 2025 Nanjing Qinheng Microelectronics Co., Ltd.
 * Attention: This software (modified or not) and binary are used for
 * microcontroller manufactured by Nanjing Qinheng Microelectronics.
 *******************************************************************************/

#include "debug.h"
#include "FreeRTOS.h"
#include "task.h"
#include "hardware.h"
#include "shared.h"
#include "la_tasks.h"

/*********************************************************************
 * @fn      main
 *
 * @brief   Main program.
 *
 * @return  none
 */
int main(void)
{
	SystemAndCoreClockUpdate();
	USART_Printf_Init(115200);
	printf("V5F SystemCoreClk:%d\r\n", SystemCoreClock);

#if (Run_Core == Run_Core_V3FandV5F)
	HSEM_FastTake(HSEM_ID0);
	HSEM_ReleaseOneSem(HSEM_ID0, 0);

    printf("FreeRTOS Kernel Version:%s\r\n",tskKERNEL_VERSION_NUMBER);

    /* Logic analyzer hardware init (USBSS/USBHS, UHSIF params, HSADC clock,
       VIO18 threshold DAC, status LEDs) */
    LogicAnalyzer_Init();

    /* create the logic analyzer tasks and start scheduling */
    LA_Tasks_Create();

    vTaskStartScheduler();

    while(1)
    {
        printf("shouldn't run at here!!\n");
    }

#elif (Run_Core == Run_Core_V3F)

#elif (Run_Core == Run_Core_V5F)
    /* Logic analyzer running stand-alone on the V5F core */
    LogicAnalyzer_Init();

    LA_Tasks_Create();

    vTaskStartScheduler();
#endif

	while(1)
	{
		printf("V5F running...\r\n");
	}
}
