/********************************** (C) COPYRIGHT *******************************
 * File Name          : la_tasks.h
 * Description        : Logic analyzer FreeRTOS tasks (V5F core).
 ********************************************************************************/
#ifndef __LA_TASKS_H
#define __LA_TASKS_H

#ifdef __cplusplus
extern "C"
{
#endif

#include "FreeRTOS.h"
#include "task.h"

/* Task configuration */
#define LA_DATA_TASK_PRIO   5
#define LA_DATA_STK_SIZE    512
#define LA_CMD_TASK_PRIO    5           /* same priority as data task: round-robin */
#define LA_CMD_STK_SIZE     256         /* with time slicing during acquisition   */

/* Task handles */
extern TaskHandle_t LaDataTask_Handler;
extern TaskHandle_t LaCmdTask_Handler;

/* Create the logic analyzer tasks (la_data_task + la_cmd_task) */
void LA_Tasks_Create(void);

#ifdef __cplusplus
}
#endif

#endif
