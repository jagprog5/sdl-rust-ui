/// give a lifetime which is a subset of the existing lifetime
pub fn reborrow<'in_life, 'out_life, T>(something: &'in_life mut T) -> &'out_life mut T
where 'in_life: 'out_life {
    &mut *something
}
