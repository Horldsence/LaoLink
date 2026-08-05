/*
 * CH32H417EVT V5F application: blink the on-board LED and print on USART8.
 */

#include <zephyr/kernel.h>
#include <zephyr/drivers/gpio.h>

#define LED0_NODE DT_ALIAS(led0)

static const struct gpio_dt_spec led = GPIO_DT_SPEC_GET(LED0_NODE, gpios);

int main(void)
{
	gpio_pin_configure_dt(&led, GPIO_OUTPUT_ACTIVE);
	printk("Hello from CH32H417EVT V5F!\n");

	while (1) {
		gpio_pin_toggle_dt(&led);
		printk("LED toggle\n");
		k_sleep(K_MSEC(500));
	}
}
