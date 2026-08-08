/********************************** (C) COPYRIGHT *******************************
 * File Name          : main.c
 * Description        : Main program body for V3F.
 *                      LaoLink (CH32H417WEU6): the V3F core wakes up the V5F
 *                      core (which runs the logic analyzer), then runs the
 *                      LaoLink system tasks. A heartbeat task on PA1 is kept
 *                      as a placeholder for the power/screen/UI functions.
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

/* Global define */
#define HEARTBEAT_TASK_PRIO     5
#define HEARTBEAT_STK_SIZE      256

/* Global Variable */
TaskHandle_t HeartbeatTask_Handler;


/*********************************************************************
 * @fn      GPIO_Toggle_INIT
 *
 * @brief   Initializes GPIOA.1 (heartbeat LED, placeholder).
 *
 * @return  none
 */
void GPIO_Toggle_INIT(void)
{
  GPIO_InitTypeDef  GPIO_InitStructure={0};

  RCC_HB2PeriphClockCmd(RCC_HB2Periph_GPIOA,ENABLE);

  GPIO_InitStructure.GPIO_Pin = GPIO_Pin_1;
  GPIO_InitStructure.GPIO_Mode = GPIO_Mode_Out_PP;
  GPIO_InitStructure.GPIO_Speed=GPIO_Speed_High;
  GPIO_Init(GPIOA, &GPIO_InitStructure);
}


/*********************************************************************
 * @fn      heartbeat_task
 *
 * @brief   Heartbeat task, toggles PA1. Placeholder for the LaoLink
 *          system functions (power control, screen, I2C/I3C scan...).
 *
 * @param  *pvParameters - task parameter (unused)
 *
 * @return  none
 */
void heartbeat_task(void *pvParameters)
{
    while(1)
    {
        GPIO_ResetBits(GPIOA, GPIO_Pin_1);
        vTaskDelay(500);
        GPIO_SetBits(GPIOA, GPIO_Pin_1);
        vTaskDelay(500);
    }
}

/*********************************************************************
 * @fn      Reset_Sharing_Data
 *
 * @brief   Initializes Sharing_Data.
 *
 * @return  none
 */
void Reset_Sharing_Data(void)
{
	for(int i = 0;i<4;i++)
	{
		Buffer_Sharing[i] = i;
	}
	Data_Sharing = 0xFFFF0000;
}


/*********************************************************************
 * @fn      main
 *
 * @brief   Main program.
 *
 * @return  none
 */
int main(void)
{
	SystemInit();
	SystemAndCoreClockUpdate();

	USART_Printf_Init(115200);
	printf("1SystemClk:%d\r\n",SystemCoreClock);
	printf( "ChipID:%08x\r\n", DBGMCU_GetCHIPID() );


#if (Run_Core == Run_Core_V3FandV5F)
	NVIC_WakeUp_V5F(Core_V5F_StartAddr);//wake up V5
	HSEM_ITConfig(HSEM_ID0, ENABLE);
    NVIC->SCTLR |= 1<<4;
	RCC_HB1PeriphClockCmd(RCC_HB1Periph_PWR,ENABLE);
	PWR_EnterSTOPMode(PWR_Regulator_ON, PWR_STOPEntry_WFE);
	HSEM_ClearFlag(HSEM_ID0);
	printf("V3F wake up\r\n");
	Reset_Sharing_Data();
	printf("FreeRTOS Kernel Version:%s\r\n",tskKERNEL_VERSION_NUMBER);

    GPIO_Toggle_INIT();
    /* create heartbeat task */
    xTaskCreate((TaskFunction_t )heartbeat_task,
                    (const char*    )"heartbeat",
                    (uint16_t       )HEARTBEAT_STK_SIZE,
                    (void*          )NULL,
                    (UBaseType_t    )HEARTBEAT_TASK_PRIO,
                    (TaskHandle_t*  )&HeartbeatTask_Handler);
    vTaskStartScheduler();

    while(1)
    {
        printf("shouldn't run at here!!\n");

    }

#elif (Run_Core == Run_Core_V3F)

#elif (Run_Core == Run_Core_V5F)
	NVIC_WakeUp_V5F(Core_V5F_StartAddr);//wake up V5
	PWR_EnterSTOPMode(PWR_Regulator_ON, PWR_STOPEntry_WFE);
	printf("V3F wake up\r\n");
#endif

	while(1)
	{
		printf("V3F running...\r\n");
		Delay_Ms(1000);
	}

}
