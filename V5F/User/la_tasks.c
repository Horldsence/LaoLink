/********************************** (C) COPYRIGHT *******************************
 * File Name          : la_tasks.c
 * Description        : Logic analyzer FreeRTOS tasks (V5F core).
 *                        - la_data_task: high-rate UHSIF/HSADC -> USB pumping
 *                        - la_cmd_task : USB command processing
 ********************************************************************************/

#include "la_tasks.h"
#include "hardware.h"
#include "ch32h417_logic.h"

/* Global Variable */
TaskHandle_t LaDataTask_Handler;
TaskHandle_t LaCmdTask_Handler;

/*********************************************************************
 * @fn      la_data_task
 *
 * @brief   High-rate data pumping task (UHSIF/HSADC -> USB).
 *          Spins with taskYIELD() while an acquisition is running so the
 *          UHSIF buffer chain never overruns; sleeps otherwise.
 *
 * @param  *pvParameters - task parameter (unused)
 *
 * @return  none
 */
static void la_data_task(void *pvParameters)
{
    while(1)
    {
        LogicAnalyzer_DataProcess();
        if(logic_adc_info.usb_trans_flag)
        {
            taskYIELD();    /* acquisition running: keep up with the UHSIF
                               buffer chain, but let la_cmd_task run */
        }
        else
        {
            vTaskDelay(1);  /* idle: nothing to pump */
        }
    }
}

/*********************************************************************
 * @fn      la_cmd_task
 *
 * @brief   USB command processing task (set parameters, start/stop, IAP).
 *
 * @param  *pvParameters - task parameter (unused)
 *
 * @return  none
 */
static void la_cmd_task(void *pvParameters)
{
    while(1)
    {
        LogicAnalyzer_CmdProcess();
        vTaskDelay(1);
    }
}

/*********************************************************************
 * @fn      LA_Tasks_Create
 *
 * @brief   Create the logic analyzer tasks.
 *
 * @return  none
 */
void LA_Tasks_Create(void)
{
    xTaskCreate((TaskFunction_t )la_data_task,
                        (const char*    )"la_data",
                        (uint16_t       )LA_DATA_STK_SIZE,
                        (void*          )NULL,
                        (UBaseType_t    )LA_DATA_TASK_PRIO,
                        (TaskHandle_t*  )&LaDataTask_Handler);

    xTaskCreate((TaskFunction_t )la_cmd_task,
                    (const char*    )"la_cmd",
                    (uint16_t       )LA_CMD_STK_SIZE,
                    (void*          )NULL,
                    (UBaseType_t    )LA_CMD_TASK_PRIO,
                    (TaskHandle_t*  )&LaCmdTask_Handler);
}
