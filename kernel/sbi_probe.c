// SPDX-License-Identifier: GPL-2.0-only
#include <linux/errno.h>
#include <linux/init.h>
#include <linux/module.h>
#include <asm/sbi.h>

static long spec_raw;
static long impl_id;
static long impl_version;
#define MAX_EXTENSIONS 32
static int extension_ids[MAX_EXTENSIONS];
static unsigned int extension_id_count;
static long extension_values[MAX_EXTENSIONS];
static unsigned int extension_value_count;

module_param(spec_raw, long, 0444);
MODULE_PARM_DESC(spec_raw, "Raw SBI specification version");
module_param(impl_id, long, 0444);
MODULE_PARM_DESC(impl_id, "SBI implementation ID");
module_param(impl_version, long, 0444);
MODULE_PARM_DESC(impl_version, "Raw SBI implementation version");
module_param_array(extension_ids, int, &extension_id_count, 0400);
MODULE_PARM_DESC(extension_ids, "SBI extension IDs to probe");
module_param_array(extension_values, long, &extension_value_count, 0444);
MODULE_PARM_DESC(extension_values, "SBI extension probe results");

static int __init sbi_probe_init(void)
{
	unsigned int i;

	spec_raw = __sbi_base_ecall(SBI_EXT_BASE_GET_SPEC_VERSION);
	impl_id = __sbi_base_ecall(SBI_EXT_BASE_GET_IMP_ID);
	impl_version = __sbi_base_ecall(SBI_EXT_BASE_GET_IMP_VERSION);

	if (spec_raw < 0 || impl_id < 0 || impl_version < 0)
		return -EOPNOTSUPP;

	extension_value_count = extension_id_count;
	for (i = 0; i < extension_id_count; i++)
		extension_values[i] = sbi_probe_extension(extension_ids[i]);

	return 0;
}

static void __exit sbi_probe_exit(void)
{
}

module_init(sbi_probe_init);
module_exit(sbi_probe_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("One-shot RISC-V SBI information probe");
