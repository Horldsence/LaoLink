/*
 * CH32H417EVT V3F waker image.
 *
 * The V3F boot core wakes the V5F application core from
 * soc_early_init_hook() when CONFIG_SOC_CH32H417_BOOT_V5F is enabled.
 * This application does not own peripherals and simply returns to idle.
 */

#include <zephyr/kernel.h>

int main(void)
{
	return 0;
}
