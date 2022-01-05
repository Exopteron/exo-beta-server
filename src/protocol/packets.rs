macro_rules! user_type {
    (VarInt) => {
        i32
    };
    (VarIntPrefixedVec <$inner:ident>) => {
        Vec<$inner>
    };
    (ShortPrefixedVec <$inner:ident>) => {
        Vec<$inner>
    };
    (LengthInferredVecU8) => {
        Vec<u8>
    };
    (Angle) => {
        f32
    };
    ($typ:ty) => {
        $typ
    };
}

macro_rules! user_type_convert_to_writeable {
    (i16, $e:expr) => {
        *$e as i16
    };
    (ShortPrefixedVec <$inner:ident>, $e:expr) => {
        ShortPrefixedVec::from($e.as_slice())
    };
    (LengthInferredVecU8, $e:expr) => {
        LengthInferredVecU8::from($e.as_slice())
    };
    ($typ:ty, $e:expr) => {
        $e
    };
}

macro_rules! packets {
    (
        $(
            $packet:ident {
                $(
                    $field:ident $typ:ident $(<$generics:ident>)?
                );* $(;)?
            } $(,)?
        )*
    ) => {
        $(
            #[derive(Debug, Clone)]
            pub struct $packet {
                $(
                    pub $field: user_type!($typ $(<$generics>)?),
                )*
            }

            #[allow(unused_imports, unused_variables)]
            impl crate::protocol::Readable for $packet {
                fn read(buffer: &mut ::std::io::Cursor<&[u8]>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<Self>
                where
                    Self: Sized
                {
                    use anyhow::Context as _;
                    $(
                        let $field = <$typ $(<$generics>)?>::read(buffer, version)
                            .context(concat!("failed to read field `", stringify!($field), "` of packet `", stringify!($packet), "`"))?
                            .into();
                    )*

                    Ok(Self {
                        $(
                            $field,
                        )*
                    })
                }
            }

            #[allow(unused_variables)]
            impl crate::protocol::Writeable for $packet {
                fn write(&self, buffer: &mut Vec<u8>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<()> {
                    $(
                        user_type_convert_to_writeable!($typ $(<$generics>)?, &self.$field).write(buffer, version)?;
                    )*
                    Ok(())
                }
            }
        )*
    };
}
impl Writeable for &Vec<u8> {
    fn write(&self, buffer: &mut Vec<u8>, version: super::ProtocolVersion) -> anyhow::Result<()> {
        (self.len() as i16).write(buffer, version)?;
        buffer.append(&mut (*self).to_owned());
        Ok(())
    }
}
macro_rules! discriminant_to_literal {
    (String, $discriminant:expr) => {
        &*$discriminant
    };
    ($discriminant_type:ident, $discriminant:expr) => {
        $discriminant.into()
    };
}

macro_rules! def_enum {
    (
        $ident:ident ($discriminant_type:ident) {
            $(
                $discriminant:literal = $variant:ident
                $(
                    {
                        $(
                            $field:ident $typ:ident $(<$generics:ident>)?
                        );* $(;)?
                    }
                )?
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub enum $ident {
            $(
                $variant
                $(
                    {
                        $(
                            $field: user_type!($typ $(<$generics>)?),
                        )*
                    }
                )?,
            )*
        }

        impl crate::protocol::Readable for $ident {
            fn read(buffer: &mut ::std::io::Cursor<&[u8]>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<Self>
                where
                    Self: Sized
            {
                use anyhow::Context as _;
                let discriminant = <$discriminant_type>::read(buffer, version)
                    .context(concat!("failed to read discriminant for enum type ", stringify!($ident)))?;

                match discriminant_to_literal!($discriminant_type, discriminant) {
                    $(
                        $discriminant => {
                            $(
                                $(
                                    let $field = <$typ $(<$generics>)?>::read(buffer, version)
                                        .context(concat!("failed to read field `", stringify!($field),
                                            "` of enum `", stringify!($ident), "::", stringify!($variant), "`"))?
                                            .into();
                                )*
                            )?

                            Ok($ident::$variant $(
                                {
                                    $(
                                        $field,
                                    )*
                                }
                            )?)
                        },
                    )*
                    _ => Err(anyhow::anyhow!(
                        concat!(
                            "no discriminant for enum `", stringify!($ident), "` matched value {:?}"
                        ), discriminant
                    ))
                }
            }
        }

        impl crate::protocol::Writeable for $ident {
            fn write(&self, buffer: &mut Vec<u8>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<()> {
                match self {
                    $(
                        $ident::$variant $(
                            {
                                $($field,)*
                            }
                        )? => {
                            let discriminant = <$discriminant_type>::from($discriminant);
                            discriminant.write(buffer, version)?;

                            $(
                                $(
                                    user_type_convert_to_writeable!($typ $(<$generics>)?, $field).write(buffer, version)?;
                                )*
                            )?
                        }
                    )*
                }
                Ok(())
            }
        }
    };
}
def_enum! {
    EntityStatusKind (i8) {
        0 = None,
        2 = EntityHurt,
        3 = EntityDead,
        4 = Unknown4,
        5 = Unknown5
    }
}
def_enum! {
    SoundEffectKind (i32) {
        1000 = Click2,
        1001 = Click1,
        1002 = BowFire,
        1003 = DoorToggle,
        1004 = Extinguish,
        1005 = RecordPlay,
        2000 = Smoke,
        2001 = BlockBreak,
    }
}
def_enum! {
    DiggingStatus (i8) {
        0 = StartedDigging,
        2 = FinishedDigging,
        4 = DropItem,
        5 = ShootArrow,
    }
}
def_enum! {
    Face (i8) {
        -1 = Invalid,
        0 = NegativeY,
        1 = PositiveY,
        2 = NegativeZ,
        3 = PositiveZ,
        4 = NegativeX,
        5 = PositiveX,
    }
}
impl Face {
    pub fn reverse(self) -> Self {
        match self {
            Self::NegativeY => Self::PositiveY,
            Self::PositiveY => Self::NegativeY,
            Self::PositiveX => Self::NegativeX,
            Self::NegativeX => Self::PositiveX,
            Self::PositiveZ => Self::NegativeZ,
            Self::NegativeZ => Self::PositiveZ,
            Face::Invalid => Self::Invalid,
        }
    }
    pub fn all_faces() -> impl Iterator<Item = Face> {
        vec![
            Face::NegativeY,
            Face::PositiveY,
            Face::NegativeZ,
            Face::PositiveZ,
            Face::NegativeX,
            Face::PositiveX,
        ]
        .into_iter()
    }
    pub fn offset(&self, mut pos: BlockPosition) -> BlockPosition {
        match self {
            Face::Invalid => (),
            Face::NegativeY => pos.y -= 1,
            Face::PositiveY => pos.y += 1,
            Face::NegativeZ => pos.z -= 1,
            Face::PositiveZ => pos.z += 1,
            Face::NegativeX => pos.x -= 1,
            Face::PositiveX => pos.x += 1,
        };
        pos
    }
}
def_enum! {
    EntityAnimationType (i8) {
        0 = NoAnimation,
        1 = SwingArm,
        2 = Damage,
        3 = LeaveBed,
        104 = Crouch,
        105 = Uncrouch,
        102 = Unknown,
    }
}
def_enum! {
    EntityEffectKind (i8) {
        1 = MoveSpeed,
        2 = MoveSlowdown,
        3 = DigSpeed,
        4 = DigSlowdown,
        5 = DamageBoost,
        6 = Heal,
        7 = Harm,
        8 = Jump,
        9 = Confusion,
        10 = Regeneration,
        11 = Resistance,
        12 = FireResistance,
        13 = WaterBreathing,
        14 = Invisibility,
        15 = Blindness,
        16 = NightVision,
        17 = Hunger,
        18 = Weakness,
        19 = Poison,
    }
}
def_enum! {
    EntityActionKind (i8) {
        1 = StartSneaking,
        2 = StopSneaking,
        3 = LeaveBed,
        4 = StartSprinting,
        5 = StopSprinting,
    }
}
macro_rules! packet_enum {
    (
        $ident:ident {
            $($id:literal = $packet:ident),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub enum $ident {
            $(
                $packet($packet),
            )*
        }

        impl $ident {
            /// Returns the packet ID of this packet.
            pub fn id(&self) -> u8 {
                match self {
                    $(
                        $ident::$packet(_) => $id,
                    )*
                }
            }
                        /// Returns the packet ID of this packet.
                        pub fn name(&self) -> String {
                            match self {
                                $(
                                    $ident::$packet(_) => stringify!($packet).to_string(),
                                )*
                            }
                        }
        }

        impl crate::protocol::Readable for $ident {
            fn read(buffer: &mut ::std::io::Cursor<&[u8]>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<Self>
            where
                Self: Sized
            {
                let packet_id = u8::read(buffer, version)?;
                match packet_id {
                    $(
                        id if id == $id => Ok($ident::$packet($packet::read(buffer, version)?)),
                    )*
                    _ => {
                        log::info!("Saw unknown packet {:x}", packet_id);
                        Err(anyhow::anyhow!("unknown packet ID {}", packet_id))
                    },
                }
            }
        }

        impl crate::protocol::Writeable for $ident {
            fn write(&self, buffer: &mut Vec<u8>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<()> {
                (self.id() as u8).write(buffer, version)?;
                match self {
                    $(
                        $ident::$packet(packet) => {
                            packet.write(buffer, version)?;
                        }
                    )*
                }
                Ok(())
            }
        }

        $(
            impl VariantOf<$ident> for $packet {
                fn discriminant_id() -> u8 { $id }

                #[allow(unreachable_patterns)]
                fn destructure(e: $ident) -> Option<Self> {
                    match e {
                        $ident::$packet(p) => Some(p),
                        _ => None,
                    }
                }
            }

            impl From<$packet> for $ident {
                fn from(packet: $packet) -> Self {
                    $ident::$packet(packet)
                }
            }
        )*
    }
}

/// Trait implemented for packets which can be converted from a packet
/// enum. For example, `SpawnEntity` implements `VariantOf<ServerPlayPacket>`.
pub trait VariantOf<Enum> {
    /// Returns the unique ID used to determine whether
    /// an enum variant matches this variant.
    fn discriminant_id() -> u8;

    /// Attempts to destructure the `Enum` into this type.
    /// Returns `None` if `enum` is not the correct variant.
    fn destructure(e: Enum) -> Option<Self>
    where
        Self: Sized;
}

use std::ops::Deref;

use crate::{
    game::BlockPosition,
    protocol::io::{LengthInferredVecU8, ShortPrefixedVec},
};

use super::Writeable;

pub mod client;
pub mod server;
